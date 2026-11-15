//! Register map of the SX1276/77/78/79 in **LoRa** mode, expressed with
//! `device-driver`'s DSL. Checked against the Semtech SX1276/77/78/79 datasheet,
//! Rev. 4 (March 2015), section 6.4 "LoRa Mode Register Map".
//!
//! The LoRa modem is independent of the FSK/OOK one and reuses addresses
//! 0x01 and 0x0D..0x3F with different meanings, so it is a separate device
//! ([`LoraRegisters`]) from the FSK map in [`crate::registers`]. Which one is
//! valid depends on `LongRangeMode` (only changeable in Sleep).
//!
//! Differences that matter:
//! * The FIFO is a 256-byte addressed buffer: set `FifoAddrPtr`, then stream
//!   bytes; the pointer auto-increments.
//! * Registers `0x0D..0x3F` are only reachable here while `AccessSharedReg` is 0.
//! * Registers 0x00-0x0C, 0x40-0x42 and 0x4D are common to both modems and are
//!   repeated here so this device is self-contained.

device_driver::create_device!(
    device_name: LoraRegisters,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type BufferAddressType = u8;
            type DefaultByteOrder = BE;
        }

        /// RegFifo (0x00): FIFO data input/output, addressed by `FifoAddrPtr`.
        /// Not accessible in Sleep.
        buffer Fifo: RW = 0x00,

        /// RegOpMode (0x01)
        register OpMode {
            const ADDRESS = 0x01;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x01;

            /// LongRangeMode: `true` = LoRa, `false` = FSK/OOK. Only writable in Sleep.
            long_range_mode: bool = 7,
            /// AccessSharedReg: `true` maps the FSK registers 0x0D..0x3F into view.
            /// Clear it again after use.
            access_shared_reg: bool = 6,
            /// See the FSK map: selects LF/HF test register access.
            low_frequency_mode: bool = 3,
            mode: uint as enum LoraMode {
                Sleep = 0,
                Standby = 1,
                FsTx = 2,
                Tx = 3,
                FsRx = 4,
                RxContinuous = 5,
                RxSingle = 6,
                Cad = 7,
            } = 0..3,
        },

        /// RegFrfMsb/Mid/Lsb (0x06-0x08): carrier = value * Fstep, Fstep = 32 MHz / 2^19.
        /// Only write in Sleep or Standby.
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

            /// PaSelect: `false` = RFO pin (max +14 dBm), `true` = PA_BOOST pin (max +20 dBm).
            pa_select: bool = 7,
            /// Pmax = 10.8 + 0.6 * MaxPower dBm.
            max_power: uint = 4..7,
            /// Pout = Pmax - (15 - OutputPower) (RFO) or 17 - (15 - OutputPower) (PA_BOOST).
            output_power: uint = 0..4,
        },

        /// RegPaRamp (0x0A): PA ramp time (0000 = 3.4 ms ... 1111 = 10 us; reset 40 us).
        register PaRamp {
            const ADDRESS = 0x0A;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x09;

            pa_ramp: uint = 0..4,
        },

        /// RegOcp (0x0B): over-current protection for the PA.
        register Ocp {
            const ADDRESS = 0x0B;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x2B;

            ocp_on: bool = 5,
            /// Imax = 45 + 5 * trim mA (trim <= 15), or -30 + 10 * trim mA (15 < trim <= 27).
            ocp_trim: uint = 0..5,
        },

        /// RegLna (0x0C)
        register Lna {
            const ADDRESS = 0x0C;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x20;

            /// LnaGain: G1 = maximum ... G6 = minimum. Used as a fixed gain only while
            /// `agc_auto_on` (ModemConfig3) is off.
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
            lna_boost_lf: uint = 3..5,
            /// LnaBoostHf: `0b11` = boost on (150% LNA current).
            lna_boost_hf: uint = 0..2,
        },

        /// RegFifoAddrPtr (0x0D): SPI pointer into the FIFO data buffer.
        register FifoAddrPtr {
            const ADDRESS = 0x0D;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            value: uint = 0..8,
        },

        /// RegFifoTxBaseAddr (0x0E): where the transmit data starts in the FIFO.
        register FifoTxBaseAddr {
            const ADDRESS = 0x0E;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x80;

            value: uint = 0..8,
        },

        /// RegFifoRxBaseAddr (0x0F): where received data starts in the FIFO.
        register FifoRxBaseAddr {
            const ADDRESS = 0x0F;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            value: uint = 0..8,
        },

        /// RegFifoRxCurrentAddr (0x10): start of the last packet received.
        register FifoRxCurrentAddr {
            type Access = RO;
            const ADDRESS = 0x10;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegIrqFlagsMask (0x11): a set bit masks (disables) that interrupt.
        register IrqFlagsMask {
            const ADDRESS = 0x11;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            rx_timeout_mask: bool = 7,
            rx_done_mask: bool = 6,
            payload_crc_error_mask: bool = 5,
            valid_header_mask: bool = 4,
            tx_done_mask: bool = 3,
            cad_done_mask: bool = 2,
            fhss_change_channel_mask: bool = 1,
            cad_detected_mask: bool = 0,
        },

        /// RegIrqFlags (0x12): writing a 1 clears the flag.
        register IrqFlags {
            const ADDRESS = 0x12;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            rx_timeout: bool = 7,
            rx_done: bool = 6,
            payload_crc_error: bool = 5,
            valid_header: bool = 4,
            tx_done: bool = 3,
            cad_done: bool = 2,
            fhss_change_channel: bool = 1,
            cad_detected: bool = 0,
        },

        /// RegRxNbBytes (0x13): number of payload bytes of the latest packet.
        register RxNbBytes {
            type Access = RO;
            const ADDRESS = 0x13;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegRxHeaderCntValueMsb/Lsb (0x14-0x15): valid headers received since entering Rx.
        register RxHeaderCount {
            type Access = RO;
            const ADDRESS = 0x14;
            const SIZE_BITS = 16;

            value: uint = 0..16,
        },

        /// RegRxPacketCntValueMsb/Lsb (0x16-0x17): valid packets received since entering Rx.
        register RxPacketCount {
            type Access = RO;
            const ADDRESS = 0x16;
            const SIZE_BITS = 16;

            value: uint = 0..16,
        },

        /// RegModemStat (0x18)
        register ModemStat {
            type Access = RO;
            const ADDRESS = 0x18;
            const SIZE_BITS = 8;

            /// Coding rate of the last header received (see [`CodingRate`]).
            rx_coding_rate: uint = 5..8,
            modem_clear: bool = 4,
            header_info_valid: bool = 3,
            rx_on_going: bool = 2,
            signal_synchronized: bool = 1,
            signal_detected: bool = 0,
        },

        /// RegPktSnrValue (0x19): SNR of the last packet, two's complement, in quarter dB.
        register PktSnrValue {
            type Access = RO;
            const ADDRESS = 0x19;
            const SIZE_BITS = 8;

            value: int = 0..8,
        },

        /// RegPktRssiValue (0x1A): RSSI of the last packet = -157 + value (HF port) or -164 + value (LF port).
        register PktRssiValue {
            type Access = RO;
            const ADDRESS = 0x1A;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegRssiValue (0x1B): current RSSI = -157 + value (HF port) or -164 + value (LF port).
        register RssiValue {
            type Access = RO;
            const ADDRESS = 0x1B;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegHopChannel (0x1C)
        register HopChannel {
            type Access = RO;
            const ADDRESS = 0x1C;
            const SIZE_BITS = 8;

            /// PLL failed to lock during a TX/RX/CAD operation.
            pll_timeout: bool = 7,
            /// CRC flag from the received header (explicit header mode only).
            crc_on_payload: bool = 6,
            fhss_present_channel: uint = 0..6,
        },

        /// RegModemConfig1 (0x1D)
        register ModemConfig1 {
            const ADDRESS = 0x1D;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x72;

            /// Signal bandwidth. 250 and 500 kHz are not supported in the 169 MHz band.
            bandwidth: uint as enum Bandwidth {
                Hz7800 = 0,
                Hz10400 = 1,
                Hz15600 = 2,
                Hz20800 = 3,
                Hz31250 = 4,
                Hz41700 = 5,
                Hz62500 = 6,
                Hz125000 = 7,
                Hz250000 = 8,
                Hz500000 = 9,
                Reserved10 = 10,
                Reserved11 = 11,
                Reserved12 = 12,
                Reserved13 = 13,
                Reserved14 = 14,
                Reserved15 = 15,
            } = 4..8,
            /// Error coding rate. In implicit header mode the receiver must be set to the sender's.
            coding_rate: uint as enum CodingRate {
                Reserved0 = 0,
                Cr45 = 1,
                Cr46 = 2,
                Cr47 = 3,
                Cr48 = 4,
                Reserved5 = 5,
                Reserved6 = 6,
                Reserved7 = 7,
            } = 1..4,
            /// ImplicitHeaderModeOn: `true` = no header (length, coding rate and CRC fixed by config).
            implicit_header: bool = 0,
        },

        /// RegModemConfig2 (0x1E)
        register ModemConfig2 {
            const ADDRESS = 0x1E;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x70;

            /// Spreading factor as log2(chips/symbol): 6..=12.
            spreading_factor: uint = 4..8,
            /// Continuous transmit (spectral analysis only).
            tx_continuous: bool = 3,
            /// Payload CRC. Implicit header: set on both sides; explicit: sender only.
            rx_payload_crc_on: bool = 2,
            /// Bits 9:8 of the Rx single-mode timeout in symbols.
            symb_timeout_msb: uint = 0..2,
        },

        /// RegSymbTimeoutLsb (0x1F): bits 7:0 of the Rx single-mode timeout, in symbols.
        register SymbTimeoutLsb {
            const ADDRESS = 0x1F;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x64;

            value: uint = 0..8,
        },

        /// RegPreambleMsb/Lsb (0x20-0x21): preamble length in symbols (plus 4.25 on air).
        register PreambleLength {
            const ADDRESS = 0x20;
            const SIZE_BITS = 16;
            const RESET_VALUE = 0x0008;

            value: uint = 0..16,
        },

        /// RegPayloadLength (0x22): payload length in bytes. Used for Tx and for
        /// implicit-header Rx. 0 is not permitted.
        register PayloadLength {
            const ADDRESS = 0x22;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x01;

            value: uint = 0..8,
        },

        /// RegMaxPayloadLength (0x23): packets whose header announces more are rejected.
        register MaxPayloadLength {
            const ADDRESS = 0x23;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0xFF;

            value: uint = 0..8,
        },

        /// RegHopPeriod (0x24): symbols between frequency hops (0 = disabled).
        register HopPeriod {
            const ADDRESS = 0x24;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            value: uint = 0..8,
        },

        /// RegFifoRxByteAddr (0x25): address of the last byte written by the Rx modem.
        register FifoRxByteAddr {
            type Access = RO;
            const ADDRESS = 0x25;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegModemConfig3 (0x26)
        register ModemConfig3 {
            const ADDRESS = 0x26;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            /// Mandated when the symbol time exceeds 16 ms. Both ends must agree.
            low_data_rate_optimize: bool = 3,
            /// `true`: LNA gain from the AGC loop; `false`: from `Lna.lna_gain`.
            agc_auto_on: bool = 2,
        },

        /// RegPpmCorrection (0x27): data rate offset, used with the AFC.
        register PpmCorrection {
            const ADDRESS = 0x27;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            value: uint = 0..8,
        },

        /// RegFeiMsb/Mid/Lsb (0x28-0x2A): 20-bit two's complement estimated frequency error
        /// of the last packet: Ferror = value * 2^24 / Fxtal * BW[kHz] / 500.
        register FreqError {
            type Access = RO;
            const ADDRESS = 0x28;
            const SIZE_BITS = 24;

            value: int = 0..20,
        },

        /// RegRssiWideband (0x2C): wideband RSSI, e.g. for random number generation.
        register RssiWideband {
            type Access = RO;
            const ADDRESS = 0x2C;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegDetectOptimize (0x31)
        register DetectOptimize {
            const ADDRESS = 0x31;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0xC3;

            /// `0b011` for SF7..SF12, `0b101` for SF6.
            detection_optimize: uint = 0..3,
        },

        /// RegInvertIQ (0x33)
        register InvertIq {
            const ADDRESS = 0x33;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x27;

            /// Invert the I and Q signals.
            invert_iq: bool = 6,
        },

        /// RegDetectionThreshold (0x37): 0x0A for SF7..SF12, 0x0C for SF6.
        register DetectionThreshold {
            const ADDRESS = 0x37;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x0A;

            value: uint = 0..8,
        },

        /// RegSyncWord (0x39): 0x12 private networks (reset), 0x34 is reserved for LoRaWAN.
        register SyncWord {
            const ADDRESS = 0x39;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x12;

            value: uint = 0..8,
        },

        /// RegDioMapping1 (0x40), LoRa meaning (Table 18):
        /// DIO0: 00 RxDone, 01 TxDone, 10 CadDone. DIO1: 00 RxTimeout, 01 FhssChangeChannel, 10 CadDetected.
        /// DIO2: FhssChangeChannel. DIO3: 00 CadDone, 01 ValidHeader, 10 PayloadCrcError.
        register DioMapping1 {
            const ADDRESS = 0x40;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            dio0: uint = 6..8,
            dio1: uint = 4..6,
            dio2: uint = 2..4,
            dio3: uint = 0..2,
        },

        /// RegDioMapping2 (0x41), LoRa meaning: DIO4: 00 CadDetected, 01/10 PllLock.
        /// DIO5: 00 ModeReady, 01/10 ClkOut.
        register DioMapping2 {
            const ADDRESS = 0x41;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x00;

            dio4: uint = 6..8,
            dio5: uint = 4..6,
        },

        /// RegVersion (0x42): 0x12 on every SX1276/77/78/79.
        register Version {
            type Access = RO;
            const ADDRESS = 0x42;
            const SIZE_BITS = 8;

            value: uint = 0..8,
        },

        /// RegPaDac (0x4D): 0x04 default, 0x07 enables +20 dBm on PA_BOOST (with OutputPower = 15).
        register PaDac {
            const ADDRESS = 0x4D;
            const SIZE_BITS = 8;
            const RESET_VALUE = 0x84;

            pa_dac: uint = 0..3,
        },
    }
);
