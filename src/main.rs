use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashSet;

/// Represent a mapping from encoded letter to decoded letter.
#[derive(Clone, Default)]
struct Mapping {
    map: [Option<u8>; 26], // index map encoded letter to decoded letter
    members: u32,          // bitset of all decoded letters that are mapped
}

impl Mapping {
    pub fn get(&self, c: u8) -> Option<u8> {
        self.map[(c - b'A') as usize]
    }

    pub fn set(&self, c: u8, l: u8) -> Result<Mapping, ()> {
        let idx_c = (c - b'A') as usize;

        if self.map[idx_c].is_none() || self.map[idx_c] == Some(l) {
            let mut result = self.clone();

            result.map[idx_c] = Some(l);
            let idx_l = (l - b'a') as usize;
            result.members |= 1 << idx_l;

            Ok(result)
        } else {
            Err(())
        }
    }

    pub fn apply(&self, ciper: &[u8]) -> Vec<u8> {
        ciper
            .iter()
            .map(|c| {
                if c.is_ascii_uppercase() {
                    self.get(*c).unwrap_or(*c)
                } else {
                    *c
                }
            })
            .collect()
    }
}

fn main() {
    let ciper = b"PRCSOFQX FP QDR AFOPQ CZSPR LA JFPALOQSKR QDFP FP ZK LIU BROJZK MOLTROE";
    let max_length = ciper.split(|&b| b == b' ').map(|w| w.len()).max().unwrap();
    let words_by_length = words_by_length(max_length);

    let ciper_disk =
        crack(ciper, &words_by_length).expect("Failed to crack ciper, exhausted all possibilities");

    // Output
    println!("Result Found!");

    let decoded = ciper_disk.apply(ciper);
    println!("====================");
    println!("abcedfghijklmnopqrstuvwxyz");
    for i in 0..26 {
        let c = ciper_disk.map.iter().position(|&x| x == Some(b'a' + i));
        print!("{}", c.map_or('?', |x| (x as u8 + b'A') as char));
    }
    println!("\n");

    println!("decoed: {}", String::from_utf8_lossy(&decoded));
    println!("====================");
}

// Get a dictionary of words, partitioned by length.
fn words_by_length<'a>(max_length: usize) -> Vec<HashSet<&'a [u8]>> {
    let mut words_by_length = vec![Vec::new(); max_length + 1];

    let two_words: &[&[u8]] = &[
        b"am", b"an", b"as", b"at", b"be", b"by", b"do", b"go", b"he", b"if", b"in", b"is", b"it",
        b"me", b"my", b"no", b"of", b"on", b"or", b"so", b"to", b"up", b"us", b"we",
    ];
    let three_words: &[&[u8]] = &[
        b"all", b"and", b"any", b"are", b"boy", b"but", b"can", b"day", b"did", b"for", b"get",
        b"had", b"has", b"her", b"him", b"his", b"how", b"its", b"let", b"man", b"new", b"not",
        b"now", b"old", b"one", b"our", b"out", b"put", b"say", b"see", b"she", b"the", b"too",
        b"two", b"use", b"was", b"way", b"who", b"you",
    ];

    include_bytes!("../words.txt")
        .split(|&c| c == b'\n' || c == b'\r')
        .filter(|b| !b.is_empty() && b.len() <= max_length)
        .for_each(|w| {
            words_by_length[w.len()].push(w);
        });
    words_by_length[2] = two_words.to_vec();
    words_by_length[3] = three_words.to_vec();
    words_by_length
        .iter()
        .map(|ws| ws.iter().copied().collect::<HashSet<_>>())
        .collect()
}

fn crack(ciper: &[u8], words_by_length: &[HashSet<&[u8]>]) -> Option<Mapping> {
    let ciper_words = ciper.split(|&b| b == b' ').collect_vec();

    for k in 0u8..26 {
        println!("trying prefix of length = {}...", k);
        let result = (0u8..26)
            .permutations(k as usize)
            .par_bridge()
            .find_map_any(|prefix| {
                let mut mapping = Mapping::default();
                for (i, &l) in prefix.iter().enumerate() {
                    mapping = mapping.set(b'A' + l, b'a' + i as u8).unwrap();
                }

                for offset in 0..26 {
                    let mut fmapping = mapping.clone();
                    let mut di = offset;
                    for i in k..26 {
                        while fmapping.map[di as usize].is_some() {
                            di = (di + 1) % 26;
                        }
                        fmapping = fmapping.set(b'A' + di, b'a' + i).unwrap();
                    }

                    let valid_words = ciper_words
                        .iter()
                        .map(|x| fmapping.apply(x))
                        .all(|cw| words_by_length[cw.len()].contains(cw.as_slice()));
                    if valid_words {
                        return Some(fmapping);
                    }
                }

                return None;
            });

        if let Some(ciper_disk) = result {
            return Some(ciper_disk);
        }
    }
    return None;
}
