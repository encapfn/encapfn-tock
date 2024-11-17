use core::cell::Cell;

use kernel::debug;
use kernel::hil::digest;
use kernel::hil::time;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;

const KEY_BUFFER: [u8; 32] = [0; 32];

pub struct HmacBench<'a, const L: usize, H: digest::Digest<'a, L>, T: time::Time> {
    hmac: &'a H,
    data_slice: &'static [u8],
    add_data_rounds: usize,
    add_data_cnt: Cell<usize>,
    hash_buf: TakeCell<'static, [u8; L]>,
    time: &'a T,
    start_time: Cell<T::Ticks>,
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: time::Time>
    HmacBench<'a, L, H, T>
{
    pub fn new(
        hmac: &'a H,
        data_slice: &'static [u8],
        add_data_rounds: usize,
        hash_buf: &'static mut [u8; L],
        time: &'a T,
    ) -> Self {
        HmacBench {
            hmac,
            data_slice,
            add_data_rounds,
            add_data_cnt: Cell::new(0),
            hash_buf: TakeCell::new(hash_buf),
            time,
            start_time: Cell::new(T::Ticks::from(0)),
        }
    }

    pub fn start(&self) {
        self.start_time.set(self.time.now());
        self.hmac.set_mode_hmacsha256(&KEY_BUFFER).unwrap();
        //debug!("Set HMAC mode with key!");
        self.add_data_iter();
    }

    fn add_data_iter(&self) {
        if self.add_data_cnt.get() < self.add_data_rounds {
            self.hmac.add_data(SubSlice::new(self.data_slice)).unwrap();
        } else {
            //panic!("Add data done!");
            self.hmac.run(self.hash_buf.take().unwrap()).unwrap();
        }
    }
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: time::Time>
    digest::ClientData<L> for HmacBench<'a, L, H, T>
{
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        self.add_data_cnt.set(self.add_data_cnt.get() + 1);
        self.add_data_iter();
    }

    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSliceMut<'static, u8>) {
        unimplemented!();
    }
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: time::Time>
    digest::ClientHash<L> for HmacBench<'a, L, H, T>
{
    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        let end_time = self.time.now();
        debug!(
            "Hash done: {:x?}, start: {:?}, end: {:?}",
            digest,
            self.start_time.get(),
            end_time
        );
    }
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: time::Time>
    digest::ClientVerify<L> for HmacBench<'a, L, H, T>
{
    fn verification_done(&self, _result: Result<bool, ErrorCode>, _compare: &'static mut [u8; L]) {
        unimplemented!();
    }
}
