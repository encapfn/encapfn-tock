#include "ef_littlefs.h"

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".encapfn_hdr")))
encapfn_fntab[17] = {
  /* 0 */ (fnptr) lfs_mount,
  /* 1 */ (fnptr) lfs_file_opencfg,
  /* 2 */ (fnptr) lfs_file_read,
  /* 3 */ (fnptr) lfs_file_rewind,
  /* 4 */ (fnptr) lfs_file_write,
  /* 5 */ (fnptr) lfs_unmount,
  /* 6 */ (fnptr) lfs_format,
  /* 7 */ (fnptr) read,
  /* 8 */ (fnptr) prog,
  /* 9 */ (fnptr) erase,
  /* 10 */ (fnptr) sync,
  /* 11 */ (fnptr) getFilledCFG,
  /* 12 */ (fnptr) getFileConfigBufferAddr,
  /* 13 */ (fnptr) getEmptyFilesystem,
  /* 14 */ (fnptr) getEmptyFileType,
  /* 15 */ (fnptr) lfs_remove,
  /* 16 */ (fnptr) lfs_file_close,
};

__attribute__ ((section (".encapfn_hdr")))
const size_t encapfn_fntab_length = sizeof(encapfn_fntab);
