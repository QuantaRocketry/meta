device_driver::create_device!(
    device_name: Lsm6ds3Registers,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }

        register FunctionConfig {
            type Access = RW;
            const ADDRESS = 0x01;
            const SIZE_BITS = 8;

            function_cfg_en: bool = 0,
            function_cfg_en_b: bool = 2,
        },

        register SensorSyncTimeFrame {
            type Access = RW;
            const ADDRESS = 0x04;
            const SIZE_BITS = 8;

            time_frame: uint as enum SensorSyncTimeFrame {
                Zero = default,
                Ms500,
                Ms1000,
                Ms1500,
                Ms2000,
                Ms2500,
                Ms3000,
                Ms3500,
                Ms4000,
                Ms4500,
                Ms5000,
                // Pad the remaining 4-bit values
                Reserved11 = 11,
                Reserved12 = 12,
                Reserved13 = 13,
                Reserved14 = 14,
                Reserved15 = 15,
            } = 0..4,
        },

        register SensorSyncResolutionRatio {
            type Access = RW;
            const ADDRESS = 0x05;
            const SIZE_BITS = 8;

            system_mode: uint as enum SensorSyncResolutionRatio {
                Ratio2_11 = default,
                Ratio2_12,
                Ratio2_13,
                Ratio2_14,
            } = 0..2,
        },

        register FifoCtrl1And2 {
            type Access = RW;
            type ByteOrder = LE;
            const ADDRESS = 0x06;
            const SIZE_BITS = 16;

            /// FIFO threshold / watermark level setting (12 bits)
            watermark: uint = 0..12,

            /// Pedometer/timestamp data ready routing to FIFO
            timer_pedo_fifo_drdy: bool = 14,

            /// Pedometer/timestamp write mode to FIFO enable
            timer_pedo_fifo_en: bool = 15,
        },

        /// Device identification register, always reads 0x6A
        register WhoAmI {
            type Access = RO;
            const ADDRESS = 0x0F;
            const SIZE_BITS = 8;
            value: uint = 0..8,
        },

        /// Linear acceleration sensor control register 1
        register Ctrl1Xl {
            type Access = RW;
            const ADDRESS = 0x10;
            const SIZE_BITS = 8;

            /// Output data rate and power mode selection
            odr_xl: uint as enum AccelOdr {
                PowerDown = 0,
                Hz12_5 = 1,
                Hz26 = 2,
                Hz52 = 3,
                Hz104 = 4,
                Hz208 = 5,
                Hz416 = 6,
                Hz833 = 7,
                Hz1660 = 8,
                Hz3330 = 9,
                Hz6660 = 10,
                // Pad the remaining 4-bit values
                Reserved11 = 11,
                Reserved12 = 12,
                Reserved13 = 13,
                Reserved14 = 14,
                Reserved15 = 15,
            } = 4..8,

            /// Accelerometer full-scale selection
            fs_xl: uint as enum AccelFullScale {
                G2 = 0,
                G16 = 1,
                G4 = 2,
                G8 = 3,
            } = 2..4,

            /// Anti-aliasing filter bandwidth selection
            bw_xl: uint as enum AccelBandwidth {
                Bw400Hz = 0,
                Bw200Hz = 1,
                Bw100Hz = 2,
                Bw50Hz = 3,
            } = 0..2,
        },

        /// Angular rate sensor control register 2
        register Ctrl2G {
            type Access = RW;
            const ADDRESS = 0x11;
            const SIZE_BITS = 8;

            /// Gyroscope output data rate selection
            odr_g: uint as enum GyroOdr {
                PowerDown = 0,
                Hz12_5 = 1,
                Hz26 = 2,
                Hz52 = 3,
                Hz104 = 4,
                Hz208 = 5,
                Hz416 = 6,
                Hz833 = 7,
                Hz1660 = 8,
                // Pad the remaining 4-bit values
                Reserved9 = 9,
                Reserved10 = 10,
                Reserved11 = 11,
                Reserved12 = 12,
                Reserved13 = 13,
                Reserved14 = 14,
                Reserved15 = 15,
            } = 4..8,

            /// Gyroscope full-scale selection
            fs_g: uint as enum GyroFullScale {
                Dps250 = 0,
                Dps500 = 1,
                Dps1000 = 2,
                Dps2000 = 3,
            } = 2..4,

            /// Gyroscope full-scale at 125 dps
            fs_125: bool = 1,
        },

        /// Control register 3 (Device configuration)
        register Ctrl3C {
            type Access = RW;
            const ADDRESS = 0x12;
            const SIZE_BITS = 8;

            boot: bool = 7,
            bdu: bool = 6,
            h_lactive: bool = 5,
            pp_od: bool = 4,
            sim: bool = 3,
            if_inc: bool = 2,
            ble: bool = 1,
            sw_reset: bool = 0,
        },

        /// Control register 4
        register Ctrl4C {
            type Access = RW;
            const ADDRESS = 0x13;
            const SIZE_BITS = 8;

            den_xl_en: bool = 7,
            sleep: bool = 6,
            int2_on_int1: bool = 5,
            den_drdy_int1: bool = 4,
            drdy_mask: bool = 3,
            i2c_disable: bool = 2,
            type_el: bool = 1,
        },

        /// Control register 5
        register Ctrl5C {
            type Access = RW;
            const ADDRESS = 0x14;
            const SIZE_BITS = 8;

            rounding: uint = 5..8,
            den_lh: bool = 4,
            st_g: uint = 2..4,
            st_xl: uint = 0..2,
        },

        /// Control register 6
        register Ctrl6C {
            type Access = RW;
            const ADDRESS = 0x15;
            const SIZE_BITS = 8;

            trig_xl: bool = 7,
            den_xl_g: bool = 6,
            xl_hm_mode: bool = 5,
            lvlen_reg: bool = 4,
            user_off_w: bool = 3,
            ftype: uint = 0..2,
        },

        /// Control register 7
        register Ctrl7G {
            type Access = RW;
            const ADDRESS = 0x16;
            const SIZE_BITS = 8;

            g_hm_mode: bool = 7,
            hp_en_g: bool = 6,
            hpm_g: uint = 4..6,
            rounding_status: bool = 2,
        },

        /// Control register 8
        register Ctrl8Xl {
            type Access = RW;
            const ADDRESS = 0x17;
            const SIZE_BITS = 8;

            lpf2_xl_en: bool = 7,
            hp_slope_xl_en: bool = 6,
            hpm_xl: uint = 4..6,
            hp_ref_mode: bool = 3,
            input_composite: bool = 2,
        },

        /// Control register 9
        register Ctrl9Xl {
            type Access = RW;
            const ADDRESS = 0x18;
            const SIZE_BITS = 8;

            zen_xl: bool = 5,
            yen_xl: bool = 4,
            xen_xl: bool = 3,
            soft_en: bool = 2,
        },

        /// Control register 10
        register Ctrl10C {
            type Access = RW;
            const ADDRESS = 0x19;
            const SIZE_BITS = 8;

            zen_g: bool = 5,
            yen_g: bool = 4,
            xen_g: bool = 3,
            func_en: bool = 2,
            pedo_rst_step: bool = 1,
            sign_motion_en: bool = 0,
        },

        /// Master configuration register for sensor hub
        register MasterConfig {
            type Access = RW;
            const ADDRESS = 0x1A;
            const SIZE_BITS = 8;

            drdy_on_int1: bool = 7,
            data_valid_sel_fifo: bool = 6,
            start_config: bool = 4,
            pull_up_en: bool = 3,
            pass_through_en: bool = 2,
            iron_en: bool = 1,
            master_on: bool = 0,
        },

        /// Wake up source register
        register WakeUpSrc {
            type Access = RO;
            const ADDRESS = 0x1B;
            const SIZE_BITS = 8;

            ff_ia: bool = 5,
            sleep_state_ia: bool = 4,
            wu_ia: bool = 3,
            x_wu: bool = 2,
            y_wu: bool = 1,
            z_wu: bool = 0,
        },

        /// Tap source register
        register TapSrc {
            type Access = RO;
            const ADDRESS = 0x1C;
            const SIZE_BITS = 8;

            tap_ia: bool = 6,
            single_tap: bool = 5,
            double_tap: bool = 4,
            tap_sign: bool = 3,
            x_tap: bool = 2,
            y_tap: bool = 1,
            z_tap: bool = 0,
        },

        /// 6D orientation source register
        register D6dSrc {
            type Access = RO;
            const ADDRESS = 0x1D;
            const SIZE_BITS = 8;

            den_drdy: bool = 7,
            d6d_ia: bool = 6,
            zh: bool = 5,
            zl: bool = 4,
            yh: bool = 3,
            yl: bool = 2,
            xh: bool = 1,
            xl: bool = 0,
        },

        /// Primary status register
        register StatusReg {
            type Access = RO;
            const ADDRESS = 0x1E;
            const SIZE_BITS = 8;

            tda: bool = 2,
            gda: bool = 1,
            xlda: bool = 0,
        },

        /// Temperature output data register (16-bit)
        register OutTemp {
            type Access = RO;
            type ByteOrder = LE;
            const ADDRESS = 0x20;
            const SIZE_BITS = 16;
            value: int = 0..16,
        },

        /// Gyroscope X, Y, and Z axes output block (48-bit / 6 bytes)
        register OutG {
            type Access = RO;
            type ByteOrder = LE;
            const ADDRESS = 0x22; // Starts at OUTX_L_G
            const SIZE_BITS = 48;

            x: int = 0..16,
            y: int = 16..32,
            z: int = 32..48,
        },

        /// Accelerometer X, Y, and Z axes output block (48-bit / 6 bytes)
        register OutXl {
            type Access = RO;
            type ByteOrder = LE;
            const ADDRESS = 0x28; // Starts at OUTX_L_XL
            const SIZE_BITS = 48;

            x: int = 0..16,
            y: int = 16..32,
            z: int = 32..48,
        },
    }
);
