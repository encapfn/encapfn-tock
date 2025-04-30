#include "ef_littlefs.h"


uint8_t files[TOTAL_SIZE] = {};


int read(const struct lfs_config * cfg, lfs_block_t block, lfs_off_t off, void * data_out, lfs_size_t siz)
{
    size_t addr = ((block * BLOCK_SIZE) + off);
    // printf("Read Addr: %lu\n", addr);
    // printf("Read Block: %lu\n", block);
    // printf("Read Off: %lu\n", off);
    if (addr + siz > TOTAL_SIZE)
    {
        return -10;
    }

    uint8_t * data_loc = files + addr;
    for (size_t i = 0; i < siz; i++)
    {
        *(((uint8_t*)data_out) + i) = *(data_loc + i);
    }

    return 0;
}

int prog(const struct lfs_config * cfg, lfs_block_t block, lfs_off_t off, const void * data, lfs_size_t siz)
{
    size_t addr = ((block * BLOCK_SIZE) + off);
    if (addr + siz > TOTAL_SIZE)
    {
        return -10;
    }

    uint8_t * data_loc = files + addr;
    for (size_t i = 0; i < siz; i++)
    {
        *(data_loc + i) = *(((uint8_t*)data) + i);
    }

    return 0;
}

int erase(const struct lfs_config * cfg, lfs_block_t block)
{
    size_t addr = (block * BLOCK_SIZE);
    uint8_t * data_loc = files + addr;
    for (size_t i = 0; i < BLOCK_SIZE; i++)
    {
        *(data_loc + i) = 0;
    }
    return 0;
}

int sync(const struct lfs_config * cfg)
{
    
    return 0;
}

void nop()
{
    return;
}


uint8_t readbuf[CACHE_SIZE];
uint8_t progbuffer[CACHE_SIZE];
uint8_t lookaheadbuffer[CACHE_SIZE];
struct lfs_config getFilledCFG()
{
    // configuration of the filesystem is provided by this struct
    const struct lfs_config cfg = {
        // block device operations
        // These are the main reason I use this helper function (too lazy to get working in Rust)
        .read  = read,
        .prog  = prog,
        .erase = erase,
        .sync  = sync,

        // block device configuration
        .read_size = 1,
        .prog_size = 1,
        .block_size = BLOCK_SIZE,
        .block_count = BLOCK_COUNT,
        .cache_size = CACHE_SIZE,
        .lookahead_size = CACHE_SIZE,
        .block_cycles = BLOCK_CYCLES,

        
        // These are currently overwritten by the Rust code
        .read_buffer = readbuf,
        .prog_buffer = progbuffer,
        .lookahead_buffer = lookaheadbuffer,
    };

    return cfg;
}

lfs_t getEmptyFilesystem()
{
    lfs_t empty_filesystem = {};
    return empty_filesystem;
}

uint8_t filebuffer[MAX_FILES * CACHE_SIZE];
void * getFileConfigBufferAddr(int i)
{
    return &filebuffer[i * CACHE_SIZE];
}


lfs_file_t getEmptyFileType(int i)
{
    lfs_file_t empty_fileType = {};
    return empty_fileType;
}

int return100()
{
    return 100;
}