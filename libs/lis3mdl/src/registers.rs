//! Register map for the ST **LIS3MDL** 3-axis magnetometer, expressed with
//! `device-driver`'s macro DSL.
//!
//! Every register/field below is transcribed directly from the datasheet
//! (STMicroelectronics `DS9463`, Rev 7, Dec 2023):
//!  - Table 15 "Register address map" for addresses
//!  - Tables 17-40 for the bit layouts of each register
//!
//! Each on-chip register is exactly one byte, addressed exactly as in
//! Table 15. Two 8-bit registers (`_L` / `_H`) are used to represent every
//! 16-bit measurement instead of one 16-bit register: this keeps every
//! bus transaction a plain single-byte read/write (Tables 11 & 13 in the
//! datasheet), sidestepping the SUB(7) auto-increment address bit
//! entirely. The 16-bit values are recombined in `lib.rs`.

device_driver::create_device!(
    device_name: Lis3mdlRegisters,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }

        /// X hard-iron offset, low byte (Section 7.1)
        register OffsetXRegL {
            const ADDRESS = 0x05;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// X hard-iron offset, high byte (Section 7.1)
        register OffsetXRegH {
            const ADDRESS = 0x06;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// Y hard-iron offset, low byte (Section 7.2)
        register OffsetYRegL {
            const ADDRESS = 0x07;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// Y hard-iron offset, high byte (Section 7.2)
        register OffsetYRegH {
            const ADDRESS = 0x08;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// Z hard-iron offset, low byte (Section 7.3)
        register OffsetZRegL {
            const ADDRESS = 0x09;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// Z hard-iron offset, high byte (Section 7.3)
        register OffsetZRegH {
            const ADDRESS = 0x0A;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },

        /// Device identification register, always reads 0x3D (Section 7.4 / Table 16)
        register WhoAmI {
            type Access = RO;
            const ADDRESS = 0x0F;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },

        /// Table 17/18: temperature enable, XY performance mode, output data
        /// rate, FAST_ODR and self-test.
        register CtrlReg1 {
            const ADDRESS = 0x20;
            const SIZE_BITS = 8;

            /// ST: enables the magnetic self-test
            self_test: bool = 0,
            /// FAST_ODR: allow ODR > 80 Hz (see Table 19)
            fast_odr: bool = 1,
            /// DO[2:0]: output data rate when FAST_ODR = 0 (Table 21)
            output_data_rate: uint as enum OutputDataRate {
                Hz0_625,
                Hz1_25,
                Hz2_5,
                Hz5,
                Hz10,
                Hz20,
                Hz40,
                Hz80,
            } = 2..5,
            /// OM[1:0]: operating mode for the X/Y axes (Table 20)
            xy_operating_mode: uint as enum XyOperatingMode {
                LowPower             = 0b00,
                MediumPerformance    = 0b01,
                HighPerformance      = 0b10,
                UltraHighPerformance = 0b11,
            } = 5..7,
            /// TEMP_EN: enables the on-die temperature sensor
            temp_en: bool = 7,
        },

        /// Table 22/23: full scale, reboot and soft reset.
        register CtrlReg2 {
            const ADDRESS = 0x21;
            const SIZE_BITS = 8;

            /// SOFT_RST: resets config & user registers
            soft_reset: bool = 2,
            /// REBOOT: reloads trim values from non-volatile memory
            reboot: bool = 3,
            /// FS[1:0]: full-scale selection (Table 24)
            full_scale: uint as enum FullScale {
                Gauss4,
                Gauss8,
                Gauss12,
                Gauss16,
            } = 5..7,
        },

        /// Table 25/26: low-power mode, SPI wire mode and system mode.
        register CtrlReg3 {
            const ADDRESS = 0x22;
            const SIZE_BITS = 8;

            /// MD[1:0]: system operating mode (Table 27). MD = 10 and 11
            /// both mean power-down, hence the `= default` catch-all.
            system_mode: uint as enum SystemMode {
                Continuous,
                Single,
                PowerDown = default,
            } = 0..2,
            /// SIM: SPI serial interface mode (irrelevant over I2C, kept
            /// for completeness/reference)
            spi_3_wire: bool = 2,
            /// LP: forces 0.625 Hz ODR + minimum averaging (Table 26)
            low_power: bool = 5,
        },

        /// Table 28/29: Z-axis performance mode and endianness of the
        /// OUT_x registers.
        register CtrlReg4 {
            const ADDRESS = 0x23;
            const SIZE_BITS = 8;

            /// BLE: 0 = data LSb at the lower address (the default, and
            /// what this driver assumes)
            big_endian: bool = 1,
            /// OMZ[1:0]: operating mode for the Z axis (Table 30)
            z_operating_mode: uint as enum ZOperatingMode {
                LowPower,
                MediumPerformance,
                HighPerformance,
                UltraHighPerformance,
            } = 2..4,
        },

        /// Table 31/32: fast-read and block-data-update.
        register CtrlReg5 {
            const ADDRESS = 0x24;
            const SIZE_BITS = 8;

            /// BDU: output registers not updated until MSB+LSB are read
            block_data_update: bool = 6,
            /// FAST_READ: allows reading only the high byte of DATA OUT
            fast_read: bool = 7,
        },

        /// Table 33/34: data-ready / overrun flags.
        register StatusReg {
            type Access = RO;
            const ADDRESS = 0x27;
            const SIZE_BITS = 8;

            x_data_available: bool = 0,
            y_data_available: bool = 1,
            z_data_available: bool = 2,
            xyz_data_available: bool = 3,
            x_data_overrun: bool = 4,
            y_data_overrun: bool = 5,
            z_data_overrun: bool = 6,
            xyz_data_overrun: bool = 7,
        },

        // /// X-axis output, low byte (Section 7.11)
        // register OutXL {
        //     type Access = RO;
        //     const ADDRESS = 0x28;
        //     const SIZE_BITS = 8;
        //     value: uint = 0..8,
        // },
        // /// X-axis output, high byte (Section 7.11), two's complement
        // register OutXH {
        //     type Access = RO;
        //     const ADDRESS = 0x29;
        //     const SIZE_BITS = 8;
        //     value: uint = 0..8,
        // },
        // /// Y-axis output, low byte (Section 7.12)
        // register OutYL {
        //     type Access = RO;
        //     const ADDRESS = 0x2A;
        //     const SIZE_BITS = 8;
        //     value: uint = 0..8,
        // },
        // /// Y-axis output, high byte (Section 7.12), two's complement
        // register OutYH {
        //     type Access = RO;
        //     const ADDRESS = 0x2B;
        //     const SIZE_BITS = 8;
        //     value: uint = 0..8,
        // },
        // /// Z-axis output, low byte (Section 7.13)
        // register OutZL {
        //     type Access = RO;
        //     const ADDRESS = 0x2C;
        //     const SIZE_BITS = 8;
        //     value: uint = 0..8,
        // },
        // /// Z-axis output, high byte (Section 7.13), two's complement
        // register OutZH {
        //     type Access = RO;
        //     const ADDRESS = 0x2D;
        //     const SIZE_BITS = 8;
        //     value: uint = 0..8,
        // },

        /// Combined X, Y, and Z axis output for burst reads (starts at 0x28).
        /// Setting SIZE_BITS to 48 forces device-driver to pass a 6-byte buffer
        /// to your I2C interface.
        register OutXyz {
            type Access = RO;
            type ByteOrder = LE;
            const ADDRESS = 0x28; // Starts at OutXL
            const SIZE_BITS = 48; // 6 bytes total

            // Extract the 16-bit fields from the burst block.
            // device-driver automatically handles casting to i16 if you specify the type.
            x: int = 0..16,
            y: int = 16..32,
            z: int = 32..48,
        },

        /// Temperature sensor output, low byte (Section 7.14)
        register TempOutL {
            type Access = RO;
            const ADDRESS = 0x2E;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// Temperature sensor output, high byte (Section 7.14), two's complement
        register TempOutH {
            type Access = RO;
            const ADDRESS = 0x2F;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },

        /// Table 35/36: interrupt configuration.
        register IntCfg {
            const ADDRESS = 0x30;
            const SIZE_BITS = 8;

            /// IEN: enables the INT pin output
            interrupt_enable: bool = 0,
            /// LIR: 0 = latched request (cleared only by reading INT_SRC),
            /// 1 = not latched
            latch_interrupt_request: bool = 1,
            /// IEA: INT pin polarity, 0 = active low, 1 = active high
            interrupt_active_high: bool = 2,
            /// ZIEN: enable interrupt generation on the Z axis
            z_interrupt_enable: bool = 5,
            /// YIEN: enable interrupt generation on the Y axis
            y_interrupt_enable: bool = 6,
            /// XIEN: enable interrupt generation on the X axis
            x_interrupt_enable: bool = 7,
        },

        /// Table 37/38: interrupt source flags (read clears a latched INT).
        register IntSrc {
            type Access = RO;
            const ADDRESS = 0x31;
            const SIZE_BITS = 8;

            interrupt_active: bool = 0,
            measurement_range_overflow: bool = 1,
            z_below_negative_threshold: bool = 2,
            y_below_negative_threshold: bool = 3,
            x_below_negative_threshold: bool = 4,
            z_above_positive_threshold: bool = 5,
            y_above_positive_threshold: bool = 6,
            x_above_positive_threshold: bool = 7,
        },

        /// Interrupt threshold, low byte (Section 7.17)
        register IntThsL {
            const ADDRESS = 0x32;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },
        /// Interrupt threshold, high byte (Section 7.17). Only bits 0..6
        /// are used (THS8..THS14); bit 7 must stay 0.
        register IntThsH {
            const ADDRESS = 0x33;
            const SIZE_BITS = 8;
            value: uint = 0..7,
        },
    }
);
