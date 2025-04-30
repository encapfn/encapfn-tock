#! /usr/bin/env bash

set -Eeuo pipefail
SHELL_NIX="$(readlink -f ./shell.nix)"
SERIAL="/dev/ttyACM1"

echo "========== OSDI'25 Evaluation Reproduction Script: OmniglotPMP =========="

sudo chown $(whoami) /dev/ttyACM*

if [ $# -lt 1 ]; then
	REUSEOUTPUT_PREFIX=""
	SERIAL_OUTPUT_PREFIX="./osdi25-eval-reproduce-output-$(date +%s)"
	echo "Saving serial output to $SERIAL_OUTPUT_PREFIX-\${BENCH}.txt files. Pass $SERIAL_OUTPUT_PREFIX as a parameter to re-use this output for generating the tables."

	echo "==> Pre-compiling littlefs..."
	pushd third-party/littlefs/
        nix-shell --pure --run "make" "$SHELL_NIX"
	popd
else
	REUSEOUTPUT_PREFIX="$1"
	echo "Reusing existing serial outputs, prefix: ${REUSEOUTPUT_PREFIX}"
fi

# From https://unix.stackexchange.com/a/366655
printarr() { declare -n __p="$1"; for k in "${!__p[@]}"; do printf "%s=%s\n" "$k" "${__p[$k]}" ; done ;  }

declare -A RES_RAW
declare -A RES_TICKS
declare -A RES_US
declare -A RES_ITERS
declare -A RES_TICKSPERITER
declare -A RES_USPERITER
declare -A RES_USPERITER_FMT

if [ "$REUSEOUTPUT_PREFIX" == "" ]; then
	echo "==> Attempting to load FPGA bitstream"
	./opentitantool --interface cw310 fpga load-bitstream ./test_rom.bit 
fi

function parseserialline() {
	if [[ "$1" =~ (OGBENCH\[(.*)\]\:\ (TICKS\=([.0-9]+)\ US\=([.0-9]+)\ ITERS\=([.0-9]+)\ TICKSPERITER\=([.0-9]+)\ USPERITER\=([.0-9]+))) ]]; then
		echo "Found benchmark result: ${BASH_REMATCH[1]}"
		LABEL="${TEST_LABEL}-${BASH_REMATCH[2]}"
		RES_RAW["$LABEL"]="${BASH_REMATCH[3]}"
		RES_TICKS["$LABEL"]="${BASH_REMATCH[4]}"
		RES_US["$LABEL"]="${BASH_REMATCH[5]}"
		RES_ITERS["$LABEL"]="${BASH_REMATCH[6]}"
		RES_TICKSPERITER["$LABEL"]="${BASH_REMATCH[7]}"
		RES_USPERITER["$LABEL"]="${BASH_REMATCH[8]}"
		RES_USPERITER_FMT["$LABEL"]="$(printf "%13.3f us" "${BASH_REMATCH[8]}")"
	fi
}

function buildflash() {
	TEST_LABEL="$1"
	TEST_NAME="$2"
	BOARD_PATH="$3"
	FEATURES="$4"
	COMBINED_BIN_NAME="$5"
	if [ "$REUSEOUTPUT_PREFIX" == "" ]; then
		echo "==> Building and flashing binary for test $TEST_NAME"
		echo "==> Building..."
		pushd "$BOARD_PATH"
		nix-shell --pure --run "make FEATURES=\"$FEATURES\"" "$SHELL_NIX"
		popd

		echo "==> Setting up serial console port..."
		stty -F "$SERIAL" 115200

		# Read from the serial in background, piping the output to this process' fd3
		exec {SERIAL_CAPTURE_FD}< <(cat "$SERIAL")
		SERIAL_CAPTURE_PROC="$!"

		echo "==> Flashing..."
		./opentitantool \
			--interface cw310 \
			bootstrap \
			"./target/riscv32imc-unknown-none-elf/release/$COMBINED_BIN_NAME"

		echo "==> Waiting for serial output to indicate test is complete..."
		while true; do
			read <&$SERIAL_CAPTURE_FD SERIAL_LINE
			echo "SERIAL_LINE: $SERIAL_LINE"
			parseserialline "$SERIAL_LINE"
			echo "$SERIAL_LINE" >> "${SERIAL_OUTPUT_PREFIX}-${TEST_LABEL}.txt"
			if echo "$SERIAL_LINE" | grep -- "-ogbenchdone-"; then
				echo "DONE!"
				exec {SERIAL_CAPTURE_FD}>&- # close the file descriptor
				kill "$SERIAL_CAPTURE_PROC"
				break
			fi
		done
	else
		REUSEOUTPUT_FILE="${REUSEOUTPUT_PREFIX}-${TEST_LABEL}.txt"
		echo "==> Parsing pre-recorded output for test $TEST_NAME from $REUSEOUTPUT_FILE"
		if ! test -f "$REUSEOUTPUT_FILE"; then
		    echo "ERROR: file $REUSEOUTPUT_FILE does not exist, cannot proceed!"
		    exit 1
		fi
		while IFS= read -r SERIAL_LINE; do
			echo "SERIAL_LINE: $SERIAL_LINE"
			parseserialline "$SERIAL_LINE"
		done < "$REUSEOUTPUT_FILE"
	fi
}

buildflash \
	"cryptolib-native" \
	"OpenTitan CryptoLib Unsafe" \
	./examples/boards/earlgrey-cw310/ \
	"og_eval_unsafe" \
	"encapfn-earlgrey-cw310-combined.bin"

buildflash \
	"cryptolib-unchecked" \
	"OpenTitan CryptoLib Isolation Only" \
	./examples/boards/earlgrey-cw310/ \
	"og_eval_isolation_only" \
	"encapfn-earlgrey-cw310-combined.bin"

buildflash \
	"cryptolib-checked" \
	"OpenTitan CryptoLib Full Omniglot" \
	./examples/boards/earlgrey-cw310/ \
	"og_eval_full" \
	"encapfn-earlgrey-cw310-combined.bin"

buildflash \
      "lwip-unchecked" \
      "LwIP Unchecked (Unsafe + Memory Isolation Only)" \
      ./examples/lwip-boards/earlgrey-cw310/ \
      "og_eval_disable_checks" \
      "encapfn-earlgrey-cw310-lwip-combined.bin"

buildflash \
	"lwip-checked" \
	"LwIP Full Omniglot" \
	./examples/lwip-boards/earlgrey-cw310/ \
	"default" \
	"encapfn-earlgrey-cw310-lwip-combined.bin"

buildflash \
	"littlefs-unchecked" \
	"LittleFS Unchecked (Unsafe + Memory Isolation Only)" \
	./examples/littlefs-boards/earlgrey-cw310/ \
	"og_eval_disable_checks" \
	"encapfn-earlgrey-cw310-littlefs-combined.bin"

buildflash \
	"littlefs-checked" \
	"LittleFS Full Omniglot" \
	./examples/littlefs-boards/earlgrey-cw310/ \
	"default" \
	"encapfn-earlgrey-cw310-littlefs-combined.bin"

buildflash \
	"ubench-invoke" \
	"Invoke Microbenchmark" \
	./examples/ubench-boards/earlgrey-cw310/ \
	"og_eval_ubench_invoke" \
	"encapfn-earlgrey-cw310-ubench-combined.bin"

buildflash \
	"ubench-validate" \
	"Validate Microbenchmark" \
	./examples/ubench-boards/earlgrey-cw310/ \
	"og_eval_ubench_validate" \
	"encapfn-earlgrey-cw310-ubench-combined.bin"

buildflash \
	"ubench-upgrade" \
	"Upgrade Microbenchmark" \
	./examples/ubench-boards/earlgrey-cw310/ \
	"og_eval_ubench_upgrade" \
	"encapfn-earlgrey-cw310-ubench-combined.bin"

buildflash \
	"ubench-callback" \
	"Callback Microbenchmark" \
	./examples/ubench-boards/earlgrey-cw310/ \
	"og_eval_ubench_callback" \
	"encapfn-earlgrey-cw310-ubench-combined.bin"

buildflash \
	"ubench-setup" \
	"Setup Microbenchmark" \
	./examples/ubench-boards/earlgrey-cw310/ \
	"og_eval_ubench_setup" \
	"encapfn-earlgrey-cw310-ubench-combined.bin"

printarr RES_RAW

echo
echo
echo "========= TABLE 3: MICROBENCHMARKS =========="
echo

cat <<EOF
(a)            | Setup                               | Invoke                              |
|              | PMP              | MPK              | PMP              | MPK              |
|------------- +------------------+------------------+------------------+------------------|
| Unsafe       | ${RES_USPERITER_FMT["ubench-setup-setup_unsafe"]} | ---------------- | ${RES_USPERITER_FMT["ubench-invoke-invoke_unsafe"]} | ---------------- |
| Omniglot     | ${RES_USPERITER_FMT["ubench-setup-setup_ef"]} | ---------------- | ${RES_USPERITER_FMT["ubench-invoke-invoke_ef_warm"]} | ---------------- |
| Sandcrust    | ---------------- | ---------------- | ---------------- | ---------------- |
| Tock Upcall  | ---------------- | ---------------- | <indep. of OG>   | ---------------- |
EOF
echo "\
Omniglot PMP Cold-Invoke:                              ${RES_USPERITER_FMT["ubench-invoke-invoke_ef_cold"]}"

echo
cat <<EOF
(b)            | Upgrade                             | Callback                            |
| allocs / CBs | PMP              | MPK              | PMP              | MPK              |
|--------------+------------------+------------------+------------------+------------------|
|            1 | ${RES_USPERITER_FMT["ubench-upgrade-upgrade ELEMS=1"]} | ---------------- | ${RES_USPERITER_FMT["ubench-callback-callback ELEMS=1"]} | ---------------- |
|            8 | ${RES_USPERITER_FMT["ubench-upgrade-upgrade ELEMS=8"]} | ---------------- | ${RES_USPERITER_FMT["ubench-callback-callback ELEMS=8"]} | ---------------- |
|           64 | ${RES_USPERITER_FMT["ubench-upgrade-upgrade ELEMS=64"]} | ---------------- | ${RES_USPERITER_FMT["ubench-callback-callback ELEMS=64"]} | ---------------- |
EOF

echo
cat <<EOF
| (c) Validate |           64B u8 |           8kB u8 |          64B str |          8kB str |
|--------------+------------------+------------------+------------------+------------------|
|          PMP | ${RES_USPERITER_FMT["ubench-validate-validate_bytes ELEMS=64"]} | ${RES_USPERITER_FMT["ubench-validate-validate_bytes ELEMS=8192"]} | ${RES_USPERITER_FMT["ubench-validate-validate_str ELEMS=64"]} | ${RES_USPERITER_FMT["ubench-validate-validate_str ELEMS=8192"]} |
|          MPK | ---------------- | ---------------- | ---------------- | ---------------- |
EOF

echo
echo
echo "========= TABLE 2: LIBRARY EVALUATIONS =========="
echo
function compute_overhead() {
	RAW_PERCENTAGE="$(echo "scale=12; ((${1} / ${2}) - 1) * 100" | bc)"
	TRIMMED="$(printf "(+%.3f%%)" "$RAW_PERCENTAGE")"
	PADDED="$(printf "%-12s" "$TRIMMED")"
	echo "$PADDED"
}

CRYPTOLIB_UNSAFE="${RES_USPERITER_FMT["cryptolib-native-Unsafe ELEMS=(512, 8)"]}"
CRYPTOLIB_ISOLATION="${RES_USPERITER_FMT["cryptolib-unchecked-IsolationOnly ELEMS=(512, 8)"]}"
CRYPTOLIB_OMNIGLOT="${RES_USPERITER_FMT["cryptolib-checked-Full ELEMS=(512, 8)"]}"
CRYPTOLIB_ISOLATION_OVERHEAD="$(\
	compute_overhead \
		"${RES_USPERITER["cryptolib-unchecked-IsolationOnly ELEMS=(512, 8)"]}" \
		"${RES_USPERITER["cryptolib-native-Unsafe ELEMS=(512, 8)"]}")"
CRYPTOLIB_OMNIGLOT_OVERHEAD="$(\
	compute_overhead \
		"${RES_USPERITER["cryptolib-checked-Full ELEMS=(512, 8)"]}" \
		"${RES_USPERITER["cryptolib-unchecked-IsolationOnly ELEMS=(512, 8)"]}")"

LITTLEFS_UNSAFE="${RES_USPERITER_FMT["littlefs-unchecked-ef_mock_unchecked ELEMS=1024"]}"
LITTLEFS_ISOLATION="${RES_USPERITER_FMT["littlefs-unchecked-ef_mpk_unchecked ELEMS=1024"]}"
LITTLEFS_OMNIGLOT="${RES_USPERITER_FMT["littlefs-checked-ef_mpk_checked ELEMS=1024"]}"
LITTLEFS_ISOLATION_OVERHEAD="$(\
	compute_overhead \
		"${RES_USPERITER["littlefs-unchecked-ef_mpk_unchecked ELEMS=1024"]}" \
		"${RES_USPERITER["littlefs-unchecked-ef_mock_unchecked ELEMS=1024"]}")"
LITTLEFS_OMNIGLOT_OVERHEAD="$(\
	compute_overhead \
		"${RES_USPERITER["littlefs-checked-ef_mpk_checked ELEMS=1024"]}" \
		"${RES_USPERITER["littlefs-unchecked-ef_mpk_unchecked ELEMS=1024"]}")"

LWIP_UNSAFE="${RES_USPERITER_FMT["lwip-unchecked-ef_mock_unchecked"]}"
LWIP_ISOLATION="${RES_USPERITER_FMT["lwip-unchecked-ef_mpk_unchecked"]}"
LWIP_OMNIGLOT="${RES_USPERITER_FMT["lwip-checked-ef_mpk_checked"]}"
LWIP_ISOLATION_OVERHEAD="$(\
	compute_overhead \
		"${RES_USPERITER["lwip-unchecked-ef_mpk_unchecked"]}" \
		"${RES_USPERITER["lwip-unchecked-ef_mock_unchecked"]}")"
LWIP_OMNIGLOT_OVERHEAD="$(\
	compute_overhead \
		"${RES_USPERITER["lwip-checked-ef_mpk_checked"]}" \
		"${RES_USPERITER["lwip-unchecked-ef_mpk_unchecked"]}")"

cat <<EOF
| Library   | RT  |    unsafe        |          isolation only       |            Omniglot           |
|-----------+-----+------------------+-------------------------------+-------------------------------|
| CryptoLib | PMP | ${CRYPTOLIB_UNSAFE} | ${CRYPTOLIB_ISOLATION} ${CRYPTOLIB_ISOLATION_OVERHEAD} | ${CRYPTOLIB_OMNIGLOT} ${CRYPTOLIB_OMNIGLOT_OVERHEAD} |
| LittleFS  | PMP | ${LITTLEFS_UNSAFE} | ${LITTLEFS_ISOLATION} ${LITTLEFS_ISOLATION_OVERHEAD} | ${LITTLEFS_OMNIGLOT} ${LITTLEFS_OMNIGLOT_OVERHEAD} |
| LwIP      | PMP | ${LWIP_UNSAFE} | ${LWIP_ISOLATION} ${LWIP_ISOLATION_OVERHEAD} | ${LWIP_OMNIGLOT} ${LWIP_OMNIGLOT_OVERHEAD} |
EOF
