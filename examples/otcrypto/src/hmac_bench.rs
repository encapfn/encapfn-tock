use core::cell::Cell;

use kernel::debug;
use kernel::hil::digest;
use kernel::hil::time::Time;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;

const KEY_BUFFER: [u8; 32] = [0; 32];

pub fn print_result<T: Time, E: core::fmt::Debug>(
    label: &str,
    elements: Option<E>,
    measurement: (usize, T::Ticks, T::Ticks),
    time: &T,
) {
    use kernel::hil::time::{ConvertTicks, Ticks};

    let (iters, start, end) = measurement;
    assert!(end > start);
    let ticks = end.wrapping_sub(start);
    let us = time.ticks_to_us(ticks);
    kernel::debug!(
        "[{}({:?})]: {:?} ticks ({} us) for {} iters, {} ticks / iter, {} us / iter",
        label,
        elements,
        ticks,
        us,
        iters,
        (ticks.into_u32() as f32) / iters as f32,
        (us as f32) / iters as f32
    );
}

pub struct HmacBench<'a, const L: usize, H: digest::Digest<'a, L>, T: Time> {
    label: &'static str,
    hmac: &'a H,
    data_slice: &'static [u8],
    add_data_rounds: usize,
    add_data_cnt: Cell<usize>,
    hash_buf: TakeCell<'static, [u8; L]>,
    time: &'a T,
    iters: usize,
    iter_cnt: Cell<usize>,
    start_time: OptionalCell<T::Ticks>,
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: Time>
    HmacBench<'a, L, H, T>
{
    pub fn new(
        label: &'static str,
        hmac: &'a H,
        data_slice: &'static [u8],
        add_data_rounds: usize,
        hash_buf: &'static mut [u8; L],
        iters: usize,
        time: &'a T,
    ) -> Self {
        HmacBench {
            label,
            hmac,
            data_slice,
            add_data_rounds,
            add_data_cnt: Cell::new(0),
            hash_buf: TakeCell::new(hash_buf),
            iters,
            iter_cnt: Cell::new(0),
            time,
            start_time: OptionalCell::empty(),
        }
    }

    pub fn start(&self) {
        if self.start_time.is_none() {
            self.start_time.set(self.time.now());
        }

        self.hmac.set_mode_hmacsha256(&KEY_BUFFER).unwrap();
        //debug!("Set HMAC mode with key!");
        self.add_data_iter();
    }

    fn bench_iter(&self) {
        if self.iter_cnt.get() < self.iters {
            self.start();
        } else {
            let end = self.time.now();
            print_result(
                self.label,
                Some((self.data_slice.len(), self.add_data_cnt.get())),
                (self.iters, self.start_time.get().unwrap(), end),
                self.time,
            );
        }
    }

    fn add_data_iter(&self) {
        if self.add_data_cnt.get() < self.add_data_rounds {
            self.hmac.add_data(SubSlice::new(self.data_slice)).unwrap();
        } else {
            self.hmac.run(self.hash_buf.take().unwrap()).unwrap();
        }
    }
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: Time>
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

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: Time>
    digest::ClientHash<L> for HmacBench<'a, L, H, T>
{
    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        // debug!(
        //     "Hash done: {:x?}, start: {:?}, end: {:?}",
        //     digest,
        //     self.start_time.get(),
        //     end_time
        // );
        self.hash_buf.replace(digest);
        self.iter_cnt.set(self.iter_cnt.get() + 1);
        self.bench_iter();
    }
}

impl<'a, const L: usize, H: digest::Digest<'a, L> + digest::HmacSha256, T: Time>
    digest::ClientVerify<L> for HmacBench<'a, L, H, T>
{
    fn verification_done(&self, _result: Result<bool, ErrorCode>, _compare: &'static mut [u8; L]) {
        unimplemented!();
    }
}
