use core::cell::{Cell, RefCell};

use atmega_hal::pac::tc0::tccr0b::CS0_A;
use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use portable_atomic::AtomicU64;

// Possible Values:
//
// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝
const PRESCALER: CS0_A = CS0_A::PRESCALE_64;
const TIMER_COUNTS: u8 = 249;

pub struct TimeDriver {
    counter: AtomicU64,
    alarms: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

struct AlarmState {
    timestamp: Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        DRIVER.counter.load(core::sync::atomic::Ordering::Relaxed)
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        });
    }
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();
embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    counter: AtomicU64::new(1), // avoid div by zero
    alarms: Mutex::new(ALARM_STATE_NEW),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn TIMER0_COMPA() {
    DRIVER.on_interrupt();
}

impl TimeDriver {
    pub fn init(tc0: atmega_hal::pac::TC0) {
        tc0.tccr0a.write(|w| w.wgm0().ctc());
        tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS));
        tc0.tccr0b.write(|w| match PRESCALER {
            CS0_A::PRESCALE_8 => w.cs0().prescale_8(),
            CS0_A::PRESCALE_64 => w.cs0().prescale_64(),
            CS0_A::PRESCALE_256 => w.cs0().prescale_256(),
            CS0_A::PRESCALE_1024 => w.cs0().prescale_1024(),
            _ => panic!(),
        });
        tc0.timsk0.write(|w| w.ocie0a().set_bit());
    }

    #[inline(always)]
    fn on_interrupt(&self) {
        let ts = self
            .counter
            .fetch_add(1, portable_atomic::Ordering::Relaxed);

        critical_section::with(|cs| {
            let alarm_state = self.alarms.borrow(cs);
            if u64::from(ts) < alarm_state.timestamp.get() {
                self.trigger_alarm(cs);
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self
            .queue
            .borrow(cs)
            .borrow_mut()
            .next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self
                .queue
                .borrow(cs)
                .borrow_mut()
                .next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let now = self.now();
        let alarm_state = self.alarms.borrow(cs);
        if timestamp < now {
            alarm_state.timestamp.set(u64::MAX);
            return false;
        }
        alarm_state.timestamp.set(timestamp);
        true
    }
}
