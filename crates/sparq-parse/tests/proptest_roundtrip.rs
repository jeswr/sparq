use std::sync::{Arc, OnceLock};

use proptest::prelude::*;
use sparq_parse::{
    compress_chunks, decode_gzip_concat, decode_zstd_concat, decode_zstd_concat_with_dict,
    train_dictionary, Codec, CompressedSink, Mode, PreparedDict, SingleMemberGzipSink,
};

fn chunks() -> impl Strategy<Value = Vec<Vec<u8>>> {
    let chunk = prop_oneof![
        1 => Just(Vec::new()),
        30 => prop::collection::vec(any::<u8>(), 0..=1_024),
        1 => prop::collection::vec(any::<u8>(), 0..=16_384),
    ];
    // 0-chunk asymmetry tracked in sq-ue37p: CompressedSink emits an empty
    // stream, while decode_gzip_concat rejects one. Empty chunks themselves
    // remain in the strategy so empty gzip members and zstd frames are covered.
    prop_oneof![
        15 => prop::collection::vec(chunk.clone(), 1..=8),
        1 => prop::collection::vec(chunk, 1..=32),
    ]
}

fn concat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}

fn run_sink(chunks: &[Vec<u8>], codec: Codec, mode: Mode) -> Vec<Vec<u8>> {
    let mut sink = CompressedSink::new(codec, mode);
    let mut members = Vec::new();
    for chunk in chunks {
        sink.push(chunk);
        members.extend(sink.try_drain().expect("compression must succeed"));
    }
    members.extend(sink.finish().expect("compression must succeed"));
    members
}

fn run_single_member(chunks: &[Vec<u8>], level: u32, mode: Mode) -> Vec<Vec<u8>> {
    let mut sink = SingleMemberGzipSink::new(level, mode);
    let mut pieces = Vec::new();
    for chunk in chunks {
        sink.push(chunk);
        pieces.extend(sink.try_drain().expect("compression must succeed"));
    }
    pieces.extend(sink.finish().expect("compression must succeed"));
    pieces
}

fn trained_dictionary() -> &'static [u8] {
    static DICTIONARY: OnceLock<Vec<u8>> = OnceLock::new();
    DICTIONARY.get_or_init(|| {
        let samples: Vec<Vec<u8>> = (0..256)
            .map(|i| {
                format!(
                    "{{\"head\":{{\"vars\":[\"s\",\"p\",\"o\"]}},\"results\":{{\"bindings\":[{{\"s\":{{\"type\":\"uri\",\"value\":\"https://example.com/resource/{i}\"}},\"p\":{{\"type\":\"uri\",\"value\":\"https://example.com/property/{}\"}},\"o\":{{\"type\":\"literal\",\"value\":\"dictionary training value {i}\"}}}}]}}}}",
                    i % 32
                )
                .into_bytes()
            })
            .collect();
        train_dictionary(&samples, 8 * 1024).expect("fixed training corpus must train")
    })
}

fn assert_codec_roundtrip<F>(
    chunks: &[Vec<u8>],
    split_seeds: (usize, usize),
    codec: Codec,
    decode: F,
) -> Result<(), TestCaseError>
where
    F: Fn(&[u8]) -> std::io::Result<Vec<u8>>,
{
    let expected = concat(chunks);
    let serial = compress_chunks(chunks, &codec, Mode::Serial).expect("compression must succeed");
    let parallel =
        compress_chunks(chunks, &codec, Mode::Parallel).expect("compression must succeed");
    prop_assert_eq!(&serial, &parallel);

    let sink_serial = run_sink(chunks, codec.clone(), Mode::Serial);
    let sink_parallel = run_sink(chunks, codec.clone(), Mode::Parallel);
    prop_assert_eq!(&sink_serial, &sink_parallel);
    prop_assert_eq!(&serial, &sink_serial);
    prop_assert_eq!(
        decode(&concat(&serial)).expect("decode must succeed"),
        expected.clone()
    );

    let mut cuts = [
        split_seeds.0 % (chunks.len() + 1),
        split_seeds.1 % (chunks.len() + 1),
    ];
    cuts.sort_unstable();
    let [first, second] = cuts;
    let first_members =
        compress_chunks(&chunks[..first], &codec, Mode::Serial).expect("compression must succeed");
    let middle_members = compress_chunks(&chunks[first..second], &codec, Mode::Parallel)
        .expect("compression must succeed");
    let last_members = run_sink(&chunks[second..], codec.clone(), Mode::Parallel);

    let mut left_grouped = first_members.clone();
    left_grouped.extend(middle_members.clone());
    left_grouped.extend(last_members.clone());
    let mut right_grouped = first_members;
    let mut middle_and_last = middle_members;
    middle_and_last.extend(last_members);
    right_grouped.extend(middle_and_last);

    prop_assert_eq!(&left_grouped, &right_grouped);
    prop_assert_eq!(&left_grouped, &serial);
    prop_assert_eq!(
        decode(&concat(&left_grouped)).expect("left-grouped members must decode"),
        concat(&chunks[..first])
            .into_iter()
            .chain(concat(&chunks[first..second]))
            .chain(concat(&chunks[second..]))
            .collect::<Vec<_>>()
    );
    prop_assert_eq!(
        decode(&concat(&right_grouped)).expect("right-grouped members must decode"),
        expected
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // Mutation witness: reversing serial member emission in compress_chunks
    // makes the serial/parallel assertion fail and shrink to two chunks.
    #[test]
    fn compression_roundtrips_across_codecs_levels_modes_and_member_grouping(
        chunks in chunks(),
        gzip_level in 0_u32..=9,
        zstd_level in -7_i32..=22,
        split_seeds in (any::<usize>(), any::<usize>()),
    ) {
        let expected = concat(&chunks);

        assert_codec_roundtrip(
            &chunks,
            split_seeds,
            Codec::Gzip { level: gzip_level },
            decode_gzip_concat,
        )?;
        assert_codec_roundtrip(
            &chunks,
            split_seeds,
            Codec::Zstd { level: zstd_level },
            decode_zstd_concat,
        )?;

        let dict = trained_dictionary();
        let dict_codec = Codec::ZstdDict(Arc::new(PreparedDict::new(
            dict.to_vec(),
            zstd_level,
        )));
        assert_codec_roundtrip(&chunks, split_seeds, dict_codec, |wire| {
            decode_zstd_concat_with_dict(wire, dict)
        })?;
        let dict_wire = concat(
            &compress_chunks(&chunks, &Codec::ZstdDict(Arc::new(PreparedDict::new(
                dict.to_vec(),
                zstd_level,
            ))), Mode::Serial)
            .expect("dictionary compression must succeed"),
        );
        prop_assert!(
            decode_zstd_concat(&dict_wire).is_err(),
            "dictionary-compressed frames must reject dictionary-less decoding"
        );

        let single_serial = run_single_member(&chunks, gzip_level, Mode::Serial);
        let single_parallel = run_single_member(&chunks, gzip_level, Mode::Parallel);
        prop_assert_eq!(&single_serial, &single_parallel);
        let single_wire = concat(&single_serial);
        prop_assert_eq!(
            decode_gzip_concat(&single_wire).expect("single gzip member must decode"),
            expected.clone()
        );

        let split = split_seeds.0 % (chunks.len() + 1);
        let mut grouped_wire = concat(&run_single_member(
            &chunks[..split],
            gzip_level,
            Mode::Serial,
        ));
        grouped_wire.extend(concat(&run_single_member(
            &chunks[split..],
            gzip_level,
            Mode::Parallel,
        )));
        prop_assert_eq!(
            decode_gzip_concat(&grouped_wire).expect("concatenated gzip members must decode"),
            expected.clone()
        );

        // Mutation witness: changing the final gzip trailer byte must never
        // decode successfully to the expected payload.
        let mut corrupted = single_wire;
        *corrupted.last_mut().expect("gzip stream has a trailer") ^= 1;
        prop_assert!(!matches!(decode_gzip_concat(&corrupted), Ok(decoded) if decoded == expected));
    }
}
