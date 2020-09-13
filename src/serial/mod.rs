//! Tooling for accessing the Serial and UART ports on the uC

use core::{
    convert::TryInto,
    ffi::c_void,
    fmt::{self, Write},
    str::{self, Utf8Error},
    sync::atomic::AtomicU32,
    sync::atomic::Ordering,
};

use crate::millis;

#[cfg(feature = "usb_logging")]
#[doc(cfg(usb_logging))]
pub mod log;

#[cfg(feature = "usb_logging")]
#[doc(cfg(usb_logging))]
pub mod ansi;

extern "C" {
    /// number of bytes available in the receive buffer
    fn usb_serial_available() -> usize;
    /// discard any buffered input
    fn usb_serial_flush_input();
    /// get the next character, or -1 if nothing received
    fn usb_serial_getchar() -> i16;
    /// peek at the next character, or -1 if nothing received
    fn usb_serial_peekchar() -> i16;
    /// read a block of bytes to a buffer. returns the size read
    fn usb_serial_read(buffer: *mut c_void, size: usize) -> usize;
    /// push out any buffered output
    fn usb_serial_flush_output();
    /// write a buffer.
    fn usb_serial_write(buffer: *const c_void, size: usize) -> usize;
    /// the free bytes available in the output buffer
    fn usb_serial_write_buffer_free() -> usize;
    static usb_cdc_line_coding: [u32; 2];
    static usb_cdc_line_rtsdtr: u8;
}
const USB_SERIAL_DTR: u8 = 0x01;
const USB_SERIAL_RTS: u8 = 0x02;

/// A representation of the parity of a serial port
#[repr(u8)]
pub enum Parity {
    /// No Parity
    None = 0,
    /// Odd Parity
    Odd = 1,
    /// Even Parity
    Even = 2,
}
impl From<u8> for Parity {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::None,
            1 => Self::Odd,
            2 => Self::Even,
            x => panic!("Attempted to convert invalid value `{}` to Parity ", x),
        }
    }
}

/// A serial USB connection to a host device. Based off of the Arduino Serial class.
/// Do not create an instance of this, instad use the provided SERIAL static
///
/// # See Also
/// - [Serial - Arduino Reference](https://www.arduino.cc/reference/en/language/functions/communication/serial/)
/// - [Teensy Serial Reference](https://www.pjrc.com/teensy/td_serial.html)
pub type SERIAL = USBSerial;

/// A serial USB connection to a host device. Based off of the Arduino Serial class.
/// Do not create an instance of this, instad use the provided SERIAL static
///
/// # See Also
/// - [Serial - Arduino Reference](https://www.arduino.cc/reference/en/language/functions/communication/serial/)
/// - [Teensy Serial Reference](https://www.pjrc.com/teensy/td_serial.html)
pub struct USBSerial {}

static SERIAL_TIMEOUT: AtomicU32 = AtomicU32::new(1000);

impl USBSerial {
    /// Set the serial read in timeout
    pub fn set_timeout(timeout: u32) {
        SERIAL_TIMEOUT.store(timeout, Ordering::Relaxed);
    }

    /// Get the number of bytes (characters) available for reading from the serial port.
    /// This is data that’s already arrived and stored in the serial receive buffer
    ///
    /// # See Also
    /// - [Serial.available() - Arduino Reference](https://www.arduino.cc/reference/en/language/functions/communication/serial/available/)
    pub fn avaliable() -> usize {
        // Call the C API and directly return the data
        unsafe { usb_serial_available() }
    }

    /// Get the number of bytes (characters) available for writing in the serial buffer
    /// without blocking the write operation.
    ///
    /// # See Also
    /// - [Serial.avaliableForWrite() - Arduino Reference](https://www.arduino.cc/reference/en/language/functions/communication/serial/availableforwrite/)
    pub fn available_for_write() -> usize {
        unsafe { usb_serial_write_buffer_free() }
    }

    /// Clear the input buffer
    pub fn clear() {
        // Call into the C API
        unsafe { usb_serial_flush_input() }
    }

    /// Transmit any buffered data as soon as possible.
    /// Sometimes referred to as flush().
    ///
    // - [Serial.flush() - Arduino Reference](https://www.arduino.cc/reference/en/language/functions/communication/serial/flush/)
    pub fn send_now() {
        unsafe { usb_serial_flush_output() }
    }

    /// Returns the next byte (character) of incoming serial data without removing it from the internal serial buffer.
    /// That is, successive calls to peek() will return the same character, as will the next call to read().
    ///
    /// # See Also
    /// - [Serial.peek() - Arduino Reference](https://www.arduino.cc/reference/en/language/functions/communication/serial/peek/)
    pub fn peek() -> Option<char> {
        // Call into the C API and store the result
        let result = unsafe { usb_serial_peekchar() };
        // usb_serial_peekchar returns a -1 if there is no char to read, so return a None
        if result == -1 {
            None
        } else {
            // If there is a char to read, get it, in a u8
            let result: u8 = result.try_into().unwrap();
            // Return the u8 as a char
            Some(result as char)
        }
    }

    /// Read the baud rate setting from the PC or Mac. Communication is always
    /// performed at full USB speed. The baud rate is useful if you intend to
    /// make a USB to serial bridge, where you need to know what speed the PC
    /// intends the serial communication to use.
    pub fn baud() -> u32 {
        unsafe { usb_cdc_line_coding[0] }
    }

    /// Read the stop bits setting from the PC or Mac. USB never uses stop bits.
    pub fn stop_bits() -> u8 {
        // Read in the bytes
        let b: u8 = unsafe { usb_cdc_line_coding[1].to_be_bytes()[0] };
        // Make 0 = 1
        if b == 0 {
            1
        } else {
            b
        }
    }

    /// Read the parity type setting from the PC or Mac. USB uses CRC checking on all
    /// bulk mode data packets and automatically retransmits corrupted data, so parity
    /// bits are never used.
    pub fn parity_type() -> Parity {
        unsafe { usb_cdc_line_coding[1].to_be_bytes()[1] }.into()
    }

    /// Read the number of bits setting from the PC or Mac.
    /// USB always communicates 8 bit bytes.
    pub fn num_bits() -> u8 {
        unsafe { usb_cdc_line_coding[1].to_be_bytes()[2] }
    }

    /// Read the DTR signal state. By default, DTR is low when no software has the serial
    /// device open, and it goes high when a program opens the port. Some programs override
    /// this behavior, but for normal software you can use DTR to know when a program is
    /// using the serial port.
    pub fn dtr() -> bool {
        unsafe { usb_cdc_line_rtsdtr & USB_SERIAL_DTR != 0 }
    }

    /// Read the RTS signal state. USB includes flow control automatically, so you do not
    /// need to read this bit to know if the PC is ready to receive your data. No matter
    /// how fast you transmit, USB always manages buffers so all data is delivered reliably.
    /// However, you can cause excessive CPU usage by the receiving program is a GUI-based
    /// java application like the Arduino serial monitior!
    ///
    /// For programs that use RTS to signal some useful information, you can read it with this
    /// function.
    pub fn rts() -> bool {
        unsafe { usb_cdc_line_rtsdtr & USB_SERIAL_RTS != 0 }
    }

    /// Read in the bytes from a serial buffer for the duration of the timeout, or until the buffer is full
    pub fn read_bytes_timeout(buffer: &mut [u8]) -> usize {
        // The current count of read in bytes
        let mut count = 0usize;
        // The length of the buffer
        let length = buffer.len();
        // The start time, for timeout
        let start_millis = millis();

        loop {
            // Increment the read in bytes by the amount that had been filled into the buffer
            count += unsafe { usb_serial_read(buffer.as_ptr().add(count) as _, length - count) };

            // Break the loop if the buffer is full
            if count >= length {
                return count;
            }

            // Stop the loop if the timeout is reached
            if millis() - start_millis < SERIAL_TIMEOUT.load(Ordering::Relaxed) {
                return count;
            }
        }
    }

    /// Read in the bytes from the serial buffer in one shot without a timeout
    pub fn read_bytes(buffer: &mut [u8]) -> usize {
        // Calculate the avaliable bytes to read in by taking the minimum of
        // the bytes in the serial buffer and in the provided buffer
        let avaliable_bytes = Self::avaliable().min(buffer.len());

        unsafe { usb_serial_read(buffer.as_mut_ptr() as _, avaliable_bytes) }
    }

    /// Read in a string from the usb buffer with retrying to fill the buffer all the way
    /// (max 256 bytes)
    pub fn read_str_timeout() -> Result<Option<&'static str>, Utf8Error> {
        static mut BUFFER: [u8; 256] = [0; 256];

        let read_in = Self::read_bytes_timeout(unsafe { &mut BUFFER });

        if read_in == 0 {
            Ok(None)
        } else {
            str::from_utf8(unsafe { &BUFFER[..read_in] }).map(Some)
        }
    }

    /// Read in a string from the usb buffer without retrying to fill the buffer all the way (max 256 bytes)
    pub fn read_str() -> Result<Option<&'static str>, Utf8Error> {
        static mut BUFFER: [u8; 256] = [0; 256];

        let read_in = Self::read_bytes(unsafe { &mut BUFFER });

        if read_in == 0 {
            Ok(None)
        } else {
            str::from_utf8(unsafe { &BUFFER[..read_in] }).map(Some)
        }
    }

    /// Read in one char of data from the serial port
    pub fn read() -> Option<char> {
        // Call into the C API and store the result
        let result = unsafe { usb_serial_getchar() };
        // usb_serial_getchar returns a -1 if there is no char to read, so return a None
        if result == -1 {
            None
        } else {
            // If there is a char to read, get it, in a u8
            let result: u8 = result.try_into().unwrap();
            // Return the u8 as a char
            Some(result as char)
        }
    }

    /// Write a single char out onto the serial port, returning if the write was successful or not
    pub fn write_char(c: char) -> bool {
        // Call into the C API with an ascii byte
        unsafe { usb_serial_write(&c as *const char as _, 1) == 1 }
    }

    /// Write a whole string out onto the serial port, returning the amount of bytes successfully written out
    pub fn write(string: &str) -> usize {
        // Get the string length
        let size = string.len();
        // Get the pointer to the string
        let ptr = string.as_ptr();

        // Call the C API
        unsafe { usb_serial_write(ptr as _, size) }
    }
}

/// A ZST that can be constructed to use the write! and writeln! macros with the global SERIAL output
pub struct USBSerialWriter;

impl Write for USBSerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let len = s.len();

        if SERIAL::write(s) != len {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if SERIAL::write_char(c) {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}
