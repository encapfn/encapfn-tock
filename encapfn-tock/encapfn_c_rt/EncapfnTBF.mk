include $(EF_TOCK_BASEDIR)/encapfn_c_rt/Configuration.mk

SRCDIR    ?= .

rwildcard = $(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

CSRC      ?= $(call rwildcard,$(SRCDIR),*.c)
COBJ      := $(addprefix $(BUILDDIR)/, $(addsuffix .o, $(subst $(SRCDIR)/,,$(CSRC))))
# TODO: adapt the above rwildcard approach for ASSRC too
ASSRC     := $(foreach x, $(SRCDIR), $(wildcard $(addprefix $(x)/*,.S))) $(INIT_S)
ASOBJ     := $(addprefix $(BUILDDIR)/, $(addsuffix .S.o, $(notdir $(basename $(ASSRC)))))

.PHONY: all
all: $(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).tab

.PHONY: clean
clean:
	rm -rf build

$(BUILDDIR)/%.c.o: $(SRCDIR)/%.c*
	mkdir -p $(shell dirname "$@")
	$(CC) $(CFLAGS) -o $@ -g -O -c $<

$(BUILDDIR)/sys.o: $(EF_TOCK_BASEDIR)/encapfn_c_rt/sys.c | $(BUILDDIR)
	mkdir -p $(BUILDDIR)
	$(CC) $(CFLAGS) -o $@ -g -O -c $<

$(BUILDDIR)/init_riscv32.S.o: $(INIT_RV32I_S)
	mkdir -p $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/%.S.o: %.S*
	mkdir -p $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).elf: \
    $(COBJ) $(ASOBJ) $(BUILDDIR)/sys.o \
    $(EF_SYSTEM_LIBS) \
    $(EF_LAYOUT_LD) \
    $(EF_TOCK_BASEDIR)/encapfn_c_rt/encapfn_layout.ld \
    $(EF_LINK_OBJ)
	mkdir -p $(BUILDDIR)
	$(LD) --no-relax -o $@ $(COBJ) $(ASOBJ) $(BUILDDIR)/sys.o $(EF_LINK_OBJ) $(EF_SYSTEM_LIBS) -T$(EF_LAYOUT_LD) $(LDFLAGS)

$(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).tab: $(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).elf
	mkdir -p $(BUILDDIR)
	elf2tab --verbose --disable -o $@ -n $(EF_BIN_NAME) $<,$(ARCH)


