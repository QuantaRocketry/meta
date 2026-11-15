//! Register map of the SX1276/77/78/79 in **FSK/OOK** mode, expressed with
//! `device-driver`'s DSL. Checked against the Semtech SX1276/77/78/79 datasheet,
//! Rev. 4 (March 2015), Tables 41 and 42 (section 6.2).
//!
//! Notes on the chip that shaped this layout:
//! * Registers are one byte wide at consecutive addresses. Multi-byte values
//!   (bit rate, deviation, carrier frequency, ...) are stored **most
//!   significant byte first**, so those registers use `ByteOrder = BE`.
//! * Address 0x00 is not a register but the FIFO, a byte stream. It is a
//!   `buffer`, not a `register`.
//! * The eight sync-word bytes are one `REPEAT`ed register.
//! * Bit numbers below are the datasheet's (bit 0 = LSB).
//! * Only the registers a receiver needs are described. Add more here.
//!
//! The LoRa modem reuses the same addresses with different meanings; it is
//! intentionally not described (this crate is FSK/OOK only).

device_driver::create_device!(
    device_name: Registers,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type BufferAddressType = u8;
            type DefaultByteOrder = BE;
        }

        /// RegFifo (0x00): FIFO data input/output.
        buffer Fifo: RW = 0x00,

        /// RegOpMode (0x01)
        register OpMode {
            const ADDRESS = 0x01;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x01;

            /// LongRangeMode: `false` = FSK/OOK, `true` = LoRa. Only writable in Sleep.
            long_range_mode: bool = 7,
            /// ModulationType.
            modulation_type: uint as enum ModulationType {
                Fsk = 0,
                Ook = 1,
                Reserved2 = 2,
                Reserved3 = 3,
            } = 5..7,
            /// LowFrequencyModeOn: selects access to the low-frequency (`true`) or
            /// high-frequency (`false`) test registers from address 0x61 up. The
            /// datasheet lists its default as 1.
            low_frequency_mode: bool = 3,
            /// Transceiver mode.
            mode: uint as enum Mode {
                Sleep = 0,
                Standby = 1,
                FsTx = 2,
                Tx = 3,
                FsRx = 4,
                Rx = 5,
                Reserved6 = 6,
                Reserved7 = 7,
            } = 0..3,
        },

        /// RegBitrateMsb/Lsb (0x02-0x03): BitRate = Fxosc / value (32 MHz crystal).
        register Bitrate {
            const ADDRESS = 0x02;
            const SIZE_BITS = 16;
            const RESET_VALUE = 0x1A0B;

            value: uint = 0..16,
        },

        /// RegFdevMsb/Lsb (0x04-0x05): Fdev = value * Fstep, Fstep = Fxosc / 2^19 (61.035 Hz).
        register Fdev {
            const ADDRESS = 0x04;
            const SIZE_BITS = 16;
            const RESET_VALUE = 0x0052;

            value: uint = 0..14,
        },

        /// RegFrfMsb/Mid/Lsb (0x06-0x08): carrier = value * Fstep.
        register Frf {
            const ADDRESS = 0x06;
            const SIZE_BITS = 24;
            const RESET_VALUE = 0x6C8000;

            value: uint = 0..24,
        },

        /// RegPaConfig (0x09)
        register PaConfig {
            const ADDRESS = 0x09;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x4F;

            /// PaSelect: `false` = RFO pin, `true` = PA_BOOST pin.
            pa_select: bool = 7,
            max_power: uint = 4..7,
            output_power: uint = 0..4,
        },

        /// RegPaRamp (0x0A)
        register PaRamp {
            const ADDRESS = 0x0A;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x09;

            /// Gaussian shaping of the transmitted data (transmit only); `GaussianBt10` is BT = 1.0, etc.
            modulation_shaping: uint as enum ModulationShaping {
                None = 0,
                GaussianBt10 = 1,
                GaussianBt05 = 2,
                GaussianBt03 = 3,
            } = 5..7,
            pa_ramp: uint = 0..4,
        },

        /// RegLna (0x0C)
        register Lna {
            const ADDRESS = 0x0C;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x20;

            /// LnaGain: G1 = maximum, G2 = -6 dB, G3 = -12 dB, G4 = -24 dB, G5 = -36 dB,
            /// G6 = -48 dB. Used as a fixed gain only while AGC is off; reading it
            /// returns the gain currently in use.
            lna_gain: uint as enum LnaGain {
                Reserved0 = 0,
                G1 = 1,
                G2 = 2,
                G3 = 3,
                G4 = 4,
                G5 = 5,
                G6 = 6,
                Reserved7 = 7,
            } = 5..8,
            /// LnaBoostHf: `0b11` = boost on for the high-frequency band.
            lna_boost_hf: uint = 0..2,
        },

        /// RegRxConfig (0x0D)
        register RxConfig {
            const ADDRESS = 0x0D;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x08;

            restart_rx_on_collision: bool = 7,
            /// Write 1 to restart the receiver without waiting for PLL lock.
            restart_rx_without_pll_lock: bool = 6,
            /// Write 1 to restart the receiver and wait for PLL lock.
            restart_rx_with_pll_lock: bool = 5,
            afc_auto_on: bool = 4,
            agc_auto_on: bool = 3,
            /// RxTrigger: event that starts the AGC/AFC measurement at receiver
            /// startup and restart (Table 24). With `None`, neither AGC nor AFC runs.
            rx_trigger: uint as enum RxTrigger {
                None = 0,
                Rssi = 1,
                Reserved2 = 2,
                Reserved3 = 3,
                Reserved4 = 4,
                Reserved5 = 5,
                PreambleDetect = 6,
                RssiAndPreambleDetect = 7,
            } = 0..3,
        },

        /// RegRssiValue (0x11): RSSI in dBm = -value / 2.
        register RssiValue {
            type Access = RO;
            const ADDRESS = 0x11;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegRxBw (0x12): channel filter bandwidth = Fxosc / (mantissa * 2^(exponent + 2)).
        register RxBw {
            const ADDRESS = 0x12;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x15;

            mantissa: uint as enum RxBwMantissa {
                M16 = 0,
                M20 = 1,
                M24 = 2,
                Reserved3 = 3,
            } = 3..5,
            exponent: uint = 0..3,
        },

        /// RegAfcBw (0x13): channel filter bandwidth used during AFC. Same encoding as RxBw.
        register AfcBw {
            const ADDRESS = 0x13;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x0B;

            mantissa: uint as enum AfcBwMantissa {
                M16 = 0,
                M20 = 1,
                M24 = 2,
                Reserved3 = 3,
            } = 3..5,
            exponent: uint = 0..3,
        },

        /// RegPreambleDetect (0x1F)
        register PreambleDetect {
            const ADDRESS = 0x1F;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0xAA;

            preamble_detector_on: bool = 7,
            /// Number of preamble bytes to detect, minus one (0 = 1 byte, 1 = 2, 2 = 3; 3 is reserved).
            preamble_detector_size: uint = 5..7,
            /// Allowed number of chip errors in the detection.
            preamble_detector_tolerance: uint = 0..5,
        },

        /// RegPreambleMsb/Lsb (0x25-0x26): preamble length transmitted, in bytes.
        register PreambleSize {
            const ADDRESS = 0x25;
            const SIZE_BITS = 16;
            const RESET_VALUE = 0x0003;

            value: uint = 0..16,
        },

        /// RegSyncConfig (0x27)
        register SyncConfig {
            const ADDRESS = 0x27;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x93;

            auto_restart_rx_mode: uint as enum AutoRestartRxMode {
                Off = 0,
                OnWithoutPllRelock = 1,
                OnWithPllRelock = 2,
                Reserved3 = 3,
            } = 6..8,
            /// PreamblePolarity: `false` = 0xAA, `true` = 0x55.
            preamble_polarity_55: bool = 5,
            sync_on: bool = 4,
            /// SyncSize: number of sync bytes minus one.
            sync_size: uint = 0..3,
        },

        /// RegSyncValue1..8 (0x28-0x2F): sync word, first byte on air first.
        register SyncValue {
            const ADDRESS = 0x28;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x01;
            const REPEAT = {
                count: 8,
                stride: 1,
            };

            value: uint = 0..8,
        },

        /// RegPacketConfig1 (0x30)
        register PacketConfig1 {
            const ADDRESS = 0x30;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x90;

            /// PacketFormat: `false` = fixed length, `true` = variable length.
            variable_length: bool = 7,
            dc_free: uint as enum DcFree {
                None = 0,
                Manchester = 1,
                Whitening = 2,
                Reserved3 = 3,
            } = 5..7,
            crc_on: bool = 4,
            /// `true` keeps a packet with a bad CRC in the FIFO instead of clearing it.
            crc_auto_clear_off: bool = 3,
            address_filtering: uint = 1..3,
            /// `false` = CCITT CRC (init 0x1D0F), `true` = IBM CRC.
            crc_whitening_type: bool = 0,
        },

        /// RegPacketConfig2 (0x31)
        register PacketConfig2 {
            const ADDRESS = 0x31;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x40;

            /// DataMode: `true` = packet mode, `false` = continuous mode.
            packet_mode: bool = 6,
            io_home_on: bool = 5,
            io_home_power_frame: bool = 4,
            beacon_on: bool = 3,
            /// Bits 10:8 of the payload length.
            payload_length_msb: uint = 0..3,
        },

        /// RegPayloadLength (0x32): fixed length, or the maximum accepted length in
        /// variable-length receive.
        register PayloadLength {
            const ADDRESS = 0x32;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x40;

            value: uint = 0..8,
        },

        /// RegFifoThresh (0x35)
        register FifoThresh {
            const ADDRESS = 0x35;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x8F;

            /// TxStartCondition: `true` = start when the FIFO is not empty.
            tx_start_fifo_not_empty: bool = 7,
            fifo_threshold: uint = 0..6,
        },

        /// RegImageCal (0x3B)
        register ImageCal {
            const ADDRESS = 0x3B;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x82;

            auto_image_cal_on: bool = 7,
            /// Write 1 to start an image calibration (standby only).
            image_cal_start: bool = 6,
            /// Read-only: calibration in progress.
            image_cal_running: bool = 5,
            temp_change: bool = 3,
            temp_threshold: uint = 1..3,
            temp_monitor_off: bool = 0,
        },

        /// RegIrqFlags1 (0x3E). `rssi`, `preamble_detect` and `sync_address_match`
        /// are cleared by writing 1. `rx_ready` is set in Rx only after RSSI, AGC
        /// and AFC have run, which with a preamble RxTrigger means after a preamble.
        register IrqFlags1 {
            const ADDRESS = 0x3E;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x80;

            mode_ready: bool = 7,
            rx_ready: bool = 6,
            tx_ready: bool = 5,
            pll_lock: bool = 4,
            rssi: bool = 3,
            timeout: bool = 2,
            preamble_detect: bool = 1,
            sync_address_match: bool = 0,
        },

        /// RegIrqFlags2 (0x3F). `fifo_overrun` is cleared by writing 1 (which also flushes the FIFO).
        register IrqFlags2 {
            const ADDRESS = 0x3F;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x40;

            fifo_full: bool = 7,
            /// Set while the FIFO is empty (the datasheet's `FifoEmpty`).
            fifo_empty: bool = 6,
            fifo_level: bool = 5,
            fifo_overrun: bool = 4,
            packet_sent: bool = 3,
            payload_ready: bool = 2,
            crc_ok: bool = 1,
            low_battery: bool = 0,
        },

        /// RegDioMapping1 (0x40). Meanings depend on the mode; in packet-mode
        /// receive, DIO0 = 0 is PayloadReady.
        register DioMapping1 {
            const ADDRESS = 0x40;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            dio0: uint = 6..8,
            dio1: uint = 4..6,
            dio2: uint = 2..4,
            dio3: uint = 0..2,
        },

        /// RegVersion (0x42): 0x12 on every SX1276/77/78/79.
        register Version {
            type Access = RO;
            const ADDRESS = 0x42;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },
    }
);
