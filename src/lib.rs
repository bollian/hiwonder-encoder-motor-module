use bytemuck::must_cast_slice;
use embedded_hal::i2c::{I2c, Operation};

#[repr(u8)]
pub enum MotorType {
    /// DC motor with no encoder
    NoEncoder = 0,

    /// 6V DC motor with quadrature encoder
    TT = 1,

    /// 6V DC motor with quadrature encoder
    N20 = 2,

    /// 12V DC motor with quadrature encoder
    JGB37 = 3,
}

const BOARD_I2C_ADDR: u8 = 0x34;
const POWER_ADC_REG: u8 = 0;
const MOTOR_TYPE_REG: u8 = 0x14;
const MOTOR_ENCODER_POLARITY_REG: u8 = 0x15;
const MOTOR_PWM_REG: u8 = 0x1f;
const MOTOR_SPEED_REG: u8 = 0x33;
const ENCODER_TOTAL_REG: u8 = 0x3c;

pub fn power_nanovolts<I: I2c>(i2c: &mut I) -> Result<u16, I::Error> {
    let mut bytes = [0; 2];
    i2c.write_read(BOARD_I2C_ADDR, &[POWER_ADC_REG], &mut bytes[..])?;
    Ok(u16::from_le_bytes(bytes))
}

pub fn encoder_totals<I: I2c>(i2c: &mut I) -> Result<[u32; 4], I::Error> {
    const BUF_SIZE: usize = std::mem::size_of::<[i32; 4]>();
    let mut bytes = [0; BUF_SIZE];
    i2c.write_read(BOARD_I2C_ADDR, &[ENCODER_TOTAL_REG], &mut bytes[..])?;

    let enc1 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let enc2 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let enc3 = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let enc4 = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

    Ok([enc1, enc2, enc3, enc4])
}

pub fn set_motors_type<I: I2c>(i2c: &mut I, ty: MotorType) -> Result<(), I::Error> {
    i2c.write(BOARD_I2C_ADDR, &[MOTOR_TYPE_REG, ty as u8])
}

pub fn set_encoder_polarity<I: I2c>(i2c: &mut I, polarity: bool) -> Result<(), I::Error> {
    i2c.write(BOARD_I2C_ADDR, &[MOTOR_ENCODER_POLARITY_REG, polarity as u8])
}

/// Set the PWM duty cycles of the motors
///
/// Each motor takes a value in the range [-100, 100] to set the PWM duty cycle for that motor.
/// Negative values drive the motors in reverse.
pub fn set_pwms<I: I2c>(i2c: &mut I, speeds: [i8; 4]) -> Result<(), I::Error> {
    let bytes = must_cast_slice::<_, u8>(&speeds[..]);
    i2c.transaction(BOARD_I2C_ADDR, &mut [
        Operation::Write(&[MOTOR_PWM_REG]),
        Operation::Write(bytes),
    ])
}

/// Set the speeds of all four motors in encoder clicks
///
/// The units for these speeds is encoder clicks per 10ms. Negative values drive the motors in
/// reverse.
pub fn set_closed_loop_speeds<I: I2c>(i2c: &mut I, speeds: [i8; 4]) -> Result<(), I::Error> {
    let bytes = must_cast_slice::<_, u8>(&speeds[..]);
    i2c.transaction(BOARD_I2C_ADDR, &mut [
        Operation::Write(&[MOTOR_SPEED_REG]),
        Operation::Write(bytes),
    ])
}
