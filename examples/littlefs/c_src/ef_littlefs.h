#ifndef __EF_LITTLEFS_H__
#define __EF_LITTLEFS_H__

#include "lfs.h"

// Used


struct lfs_config getFilledCFG();

int read(const struct lfs_config * cfg, lfs_block_t block, lfs_off_t off, void * data_out, lfs_size_t siz);
int prog(const struct lfs_config * cfg, lfs_block_t block, lfs_off_t off, const void * data, lfs_size_t siz);
int erase(const struct lfs_config * cfg, lfs_block_t block);
int sync(const struct lfs_config * cfg);


// Currently Unused


lfs_t getEmptyFilesystem();
lfs_file_t getEmptyFileType(int i);
void * getFileConfigBufferAddr(int i);



#define BLOCK_SIZE 512
#define BLOCK_COUNT 16
#define TOTAL_SIZE (BLOCK_COUNT * BLOCK_SIZE)
#define BLOCK_CYCLES 500

#define CACHE_SIZE 64
#define MAX_FILES 16

#endif
