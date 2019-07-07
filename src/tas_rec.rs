use crate::recording::{Frame, Move, Recording};
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::{escaped, is_a, tag, take_until, take_while};
use nom::character::complete::{alphanumeric1, char, digit0, line_ending, one_of};
use nom::combinator::{cut, map, opt};
use nom::error::{context, convert_error, ParseError, VerboseError};
use nom::multi::{many0, many1, separated_list};
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::{Err, IResult};
use regex::Regex;
use std::io;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct TasFile {
    pub mission: String,
    pub sequences: Vec<(String, Vec<Frame>)>,
}

impl TasFile {
    pub fn from_rec(recording: Recording) -> TasFile {
        let sequences: Vec<(String, Vec<Frame>)> = vec![("Imported".into(), recording.frames)];
        TasFile {
            mission: recording.mission,
            sequences,
        }
    }

    pub fn into_rec(self) -> Recording {
        let mut frames = vec![];
        let mut seqs = self.sequences;
        for sequence in &mut seqs {
            frames.append(&mut sequence.1);
        }

        Recording {
            mission: self.mission,
            frames,
        }
    }

    pub fn parse(mut input: String) -> Result<TasFile, ()> {
        let re = Regex::new(r"//.*").map_err(|_| ())?;
        let formatted = re.replace_all(input.as_str(), "");
        match tasfile::<VerboseError<&str>>(&formatted) {
            Ok((_, tf)) => Ok(tf),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                eprintln!("{}", convert_error(&formatted, e));
                Err(())
            }
            _ => Err(()),
        }
    }

    pub fn print<T>(&self, out: &mut T) -> Result<(), io::Error>
    where
        T: Write,
    {
        out.write_fmt(format_args!("{{\n   {}\n", TasFile::escape(&self.mission)))?;

        let mut elapsed: u32 = 0;

        for seq in &self.sequences {
            out.write_fmt(format_args!("   {{\n      {}\n", TasFile::escape(&seq.0)))?;

            for frame in &seq.1 {
                elapsed += frame.delta as u32;
                let has_moves = frame.moves[0].is_some() || frame.moves[1].is_some();
                if has_moves {
                    out.write_fmt(format_args!(
                        "      moveframe {} ms //{}\n",
                        frame.delta, elapsed
                    ))?;
                    out.write_fmt(format_args!("      {{\n"))?;
                    if let Some(mv) = &frame.moves[0] {
                        out.write_fmt(format_args!(
                            "         camera ({} {} {})\n",
                            mv.yaw.unwrap_or(0f64),
                            mv.pitch.unwrap_or(0f64),
                            mv.roll.unwrap_or(0f64)
                        ))?;
                        out.write_fmt(format_args!(
                            "         move ({} {} {})\n",
                            mv.mx, mv.my, mv.mz
                        ))?;
                        out.write_fmt(format_args!(
                            "         triggers ({} {} {} {} {} {})\n",
                            mv.triggers[0] as u8,
                            mv.triggers[1] as u8,
                            mv.triggers[2] as u8,
                            mv.triggers[3] as u8,
                            mv.triggers[4] as u8,
                            mv.triggers[5] as u8
                        ))?;
                    }
                    out.write_fmt(format_args!("      }}\n"))?;
                    out.write_fmt(format_args!("      {{\n"))?;
                    if let Some(mv) = &frame.moves[1] {
                        out.write_fmt(format_args!(
                            "         camera ({} {} {})\n",
                            mv.yaw.unwrap_or(0f64),
                            mv.pitch.unwrap_or(0f64),
                            mv.roll.unwrap_or(0f64)
                        ))?;
                        out.write_fmt(format_args!(
                            "         move ({} {} {})\n",
                            mv.mx, mv.my, mv.mz
                        ))?;
                        out.write_fmt(format_args!(
                            "         triggers ({} {} {} {} {} {})\n",
                            mv.triggers[0] as u8,
                            mv.triggers[1] as u8,
                            mv.triggers[2] as u8,
                            mv.triggers[3] as u8,
                            mv.triggers[4] as u8,
                            mv.triggers[5] as u8
                        ))?;
                    }
                    out.write_fmt(format_args!("      }}\n"))?;
                } else {
                    out.write_fmt(format_args!(
                        "      frame {} ms //{}\n",
                        frame.delta, elapsed
                    ))?;
                }
            }

            out.write_fmt(format_args!("   }}\n"))?;
        }
        out.write_fmt(format_args!("}}\n"))
    }

    pub fn escape(value: &String) -> String {
        format!("\"{}\"", value.replace("\\", "\\\\").replace("\"", "\\\""))
    }
}

fn delim_context_cut<'a, E: ParseError<&'a str>, F, G, H, O1, O2, O3>(
    name: &'static str,
    before: F,
    inner: G,
    after: H,
) -> impl Fn(&'a str) -> IResult<&'a str, O2, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, E>,
    G: Fn(&'a str) -> IResult<&'a str, O2, E>,
    H: Fn(&'a str) -> IResult<&'a str, O3, E>,
{
    context(name, preceded(before, cut(terminated(inner, after))))
}

fn ws_before<'a, E: ParseError<&'a str>, F, O>(
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    preceded(opt(sp), inner)
}

fn ws_wrap<'a, E: ParseError<&'a str>, F, O>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    preceded(opt(sp), terminated(inner, opt(sp)))
}

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(i)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(is_not("\"\\"), '\\', one_of("\"n\\"))(i)
}

fn string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
    )(i)
}

fn empty_frame<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Frame>, E> {
    ws_before(map(terminated(double, preceded(sp, tag("ms"))), |ms| {
        vec![Frame {
            moves: [None, None],
            delta: ms as u16,
        }]
    }))(i)
}

fn empty_frames<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Frame>, E> {
    ws_before(map(double, |n| {
        let mut v: Vec<Frame> = Vec::with_capacity(n as usize);
        for i in 0..n as usize {
            v.push(Frame {
                moves: [None, None],
                delta: 1,
            })
        }
        v
    }))(i)
}

fn float3<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (f64, f64, f64), E> {
    delim_context_cut(
        "float3",
        char('('),
        ws_wrap(tuple((
            preceded(opt(sp), double),
            preceded(opt(sp), double),
            preceded(opt(sp), double),
        ))),
        char(')'),
    )(i)
}

fn bool6<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (bool, bool, bool, bool, bool, bool), E> {
    delim_context_cut(
        "bool6",
        char('('),
        ws_wrap(map(
            tuple((
                preceded(opt(sp), is_a("01")),
                preceded(opt(sp), is_a("01")),
                preceded(opt(sp), is_a("01")),
                preceded(opt(sp), is_a("01")),
                preceded(opt(sp), is_a("01")),
                preceded(opt(sp), is_a("01")),
            )),
            |(a, b, c, d, e, f)| {
                (
                    a.chars().nth(0).map_or(false, |ch| ch == '1'),
                    b.chars().nth(0).map_or(false, |ch| ch == '1'),
                    c.chars().nth(0).map_or(false, |ch| ch == '1'),
                    d.chars().nth(0).map_or(false, |ch| ch == '1'),
                    e.chars().nth(0).map_or(false, |ch| ch == '1'),
                    f.chars().nth(0).map_or(false, |ch| ch == '1'),
                )
            },
        )),
        char(')'),
    )(i)
}

fn move_inner<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Move, E> {
    ws_wrap(map(
        tuple((
            preceded(tag("camera"), ws_wrap(float3)),
            preceded(tag("move"), ws_wrap(float3)),
            preceded(tag("triggers"), ws_wrap(bool6)),
        )),
        |((yaw, pitch, roll), (mx, my, mz), triggers)| Move {
            yaw: Some(yaw),
            pitch: Some(pitch),
            roll: Some(roll),
            mx: mx,
            my: my,
            mz: mz,
            freelook: true,
            triggers: [
                triggers.0, triggers.1, triggers.2, triggers.3, triggers.4, triggers.5,
            ],
        },
    ))(i)
}

fn move_<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Option<Move>, E> {
    delim_context_cut("move", char('{'), ws_before(opt(move_inner)), char('}'))(i)
}

fn move_frame<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Frame>, E> {
    ws_before(map(
        separated_pair(
            terminated(double, preceded(sp, tag("ms"))),
            sp,
            separated_pair(move_, sp, move_),
        ),
        |(ms, (mv0, mv1))| {
            vec![Frame {
                moves: [mv0, mv1],
                delta: ms as u16,
            }]
        },
    ))(i)
}

fn frame<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Frame>, E> {
    alt((
        preceded(tag("frames"), context("frames", cut(empty_frames))),
        preceded(tag("frame"), context("frame", cut(empty_frame))),
        preceded(tag("moveframe"), context("moveframe", cut(move_frame))),
    ))(i)
}

fn sequence<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (String, Vec<Frame>), E> {
    delim_context_cut(
        "sequence",
        char('{'),
        ws_wrap(map(
            separated_pair(string, sp, separated_list(sp, frame)),
            |(name, mut frames)| {
                let mut collected = vec![];
                for list in &mut frames {
                    collected.append(list);
                }
                (name.into(), collected)
            },
        )),
        char('}'),
    )(i)
}

fn tasfile<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TasFile, E> {
    delim_context_cut(
        "tasfile",
        char('{'),
        map(
            tuple((ws_wrap(string), many0(ws_wrap(sequence)))),
            |(mission, sequences)| TasFile {
                mission: mission.into(),
                sequences,
            },
        ),
        char('}'),
    )(i)
}
