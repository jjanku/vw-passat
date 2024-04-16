use std::io::{BufWriter, Write};

use crate::types::Proof;

pub enum Format {
    Binary,
    Plain,
}

pub fn write_proof(writer: &mut impl Write, format: Format, proof: &Proof) {
    let mut writer = BufWriter::new(writer);

    for (step, clause) in proof {
        match format {
            Format::Binary => binary::write_lemma(&mut writer, *step, clause),
            Format::Plain => plain::write_lemma(&mut writer, *step, clause),
        };
    }
}

mod binary {
    use std::io::Write;

    use crate::types::{Clause, Lit, ProofStep};

    const BUF_SIZE: usize = (u32::BITS / 7 + 1) as usize;

    fn var_byte_encode(mut num: u32, buf: &mut [u8; BUF_SIZE]) -> &[u8] {
        let mut len = 0;
        loop {
            buf[len] = (num & 127) as u8 | 128;
            len += 1;
            num >>= 7;

            if num == 0 {
                break;
            }
        }
        buf[len - 1] &= 127;
        &buf[..len]
    }

    fn encode_lit(lit: Lit, buf: &mut [u8; BUF_SIZE]) -> &[u8] {
        let ulit = if lit > 0 { 2 * lit } else { -2 * lit + 1 } as u32;
        var_byte_encode(ulit, buf)
    }

    pub fn write_lemma(writer: &mut impl Write, step: ProofStep, clause: &Clause) {
        let step_code = match step {
            ProofStep::Add => b'a',
            ProofStep::Delete => b'd',
        };
        writer.write(&[step_code]).unwrap();

        let mut buf = [0; BUF_SIZE];
        for &lit in clause {
            let enc = encode_lit(lit, &mut buf);
            writer.write(enc).unwrap();
        }

        writer.write(&[0]).unwrap();
    }

    #[cfg(test)]
    mod tests {
        use crate::types::ProofStep;

        use super::{var_byte_encode, write_lemma, BUF_SIZE};

        #[test]
        fn var_byte_encoding() {
            fn test_encode(num: u32, bytes: &[u8]) {
                let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
                assert_eq!(var_byte_encode(num, &mut buf), bytes);
            }

            test_encode(0, &[0x00]);
            test_encode(1, &[0x01]);
            test_encode((1 << 7) - 1, &[0x7f]);
            test_encode(1 << 7, &[0x80, 0x01]);
            test_encode((1 << 8) + 2, &[0x82, 0x02]);
            test_encode((1 << 14) - 1, &[0xff, 0x7f]);
            test_encode((1 << 14) + 3, &[0x83, 0x80, 0x01]);
            test_encode((1 << 28) - 1, &[0xff, 0xff, 0xff, 0x7f]);
            test_encode((1 << 28) + 7, &[0x87, 0x80, 0x80, 0x80, 0x01]);
        }

        #[test]
        fn lemma_encoding() {
            let mut buf: Vec<u8> = vec![];
            write_lemma(&mut buf, ProofStep::Delete, &vec![-63, -8193]);
            write_lemma(&mut buf, ProofStep::Add, &vec![129, -8191]);
            assert_eq!(buf, b"\x64\x7f\x83\x80\x01\x00\x61\x82\x02\xff\x7f\x00");
        }
    }
}

mod plain {
    use std::io::Write;

    use crate::types::{Clause, ProofStep};

    pub fn write_lemma(writer: &mut impl Write, step: ProofStep, clause: &Clause) {
        let step_str = match step {
            ProofStep::Add => "",
            ProofStep::Delete => "d ",
        };
        let clause_str = clause
            .iter()
            .fold(String::new(), |str, lit| str + &lit.to_string() + " ");
        writeln!(writer, "{step_str}{clause_str}0").unwrap();
    }

    #[cfg(test)]
    mod tests {
        use crate::types::ProofStep;

        use super::write_lemma;

        #[test]
        fn lemma_encoding() {
            let mut buf: Vec<u8> = vec![];
            write_lemma(&mut buf, ProofStep::Delete, &vec![-63, -8193]);
            write_lemma(&mut buf, ProofStep::Add, &vec![129, -8191]);
            let str = std::str::from_utf8(&buf).unwrap();
            assert_eq!(str, "d -63 -8193 0\n129 -8191 0\n");
        }
    }
}
