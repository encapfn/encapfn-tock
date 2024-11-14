include $(EF_TOCK_BASEDIR)/encapfn_c_rt/Configuration.mk

SRCDIR ?= .

# SRC := $(foreach x, ./, $(wildcard $(addprefix $(x)/*,.c*)))
CSRC := $(foreach x, $(SRCDIR), $(wildcard $(addprefix $(x)/*,.c)))
COBJ := $(addprefix $(BUILDDIR)/, $(addsuffix .c.o, $(notdir $(basename $(CSRC)))))
ASSRC := $(foreach x, $(SRCDIR), $(wildcard $(addprefix $(x)/*,.S))) $(INIT_S)
ASOBJ := $(addprefix $(BUILDDIR)/, $(addsuffix .S.o, $(notdir $(basename $(ASSRC)))))

.PHONY: all
all: $(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).tab

.PHONY: clean
clean:
	rm -rf build

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/%.c.o: $(SRCDIR)/%.c* | $(BUILDDIR)
	$(CC) $(CFLAGS) -o $@ -g -O -c $<

$(BUILDDIR)/init_riscv32.S.o: $(INIT_RV32I_S) | $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/init_cortexm.S.o: $(INIT_CORTEXM_S) | $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/%.S.o: %.S* | $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).elf: \
    $(COBJ) $(ASOBJ) \
    $(NEWLIB_BASE_DIR)/$(NEWLIB_TARGET)/libc.a \
    $(NEWLIB_BASE_DIR)/$(NEWLIB_TARGET)/libm.a \
    $(EF_LAYOUT_LD) \
    $(EF_TOCK_BASEDIR)/encapfn_c_rt/encapfn_layout.ld \
    | $(BUILDDIR)
	$(LD) --no-relax -o $@ $(COBJ) $(ASOBJ) -T$(EF_LAYOUT_LD) $(LDFLAGS)

$(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).tab: $(BUILDDIR)/$(EF_TARGET)_$(EF_BIN_NAME).elf | $(BUILDDIR)
	elf2tab --verbose --minimum-footer-size 3000 --disable -o $@ -n $(EF_BIN_NAME) $<,$(ARCH)


