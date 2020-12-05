use std::cmp::Ordering;
use std::io::{self, BufRead};
use std::{env, error, fmt, fs, path, result};

#[derive(fmt::Debug)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Error {
        Error { message }
    }
}

type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::new(format!("io error:{}", e))
    }
}

#[derive(Eq, PartialEq)]
struct Seat {
    id: u32,
    code: String,
}

fn to_id(code: &str) -> Result<u32> {
    code.chars().enumerate().fold(Ok(0u32), |acc, (i, c)| {
        acc.and_then(|id| {
            let mask = 1u32 << (9 - i);
            match c {
                'F' | 'L' => Ok(id),
                'B' | 'R' => Ok(id | mask),
                _ => Err(Error::new(format!("{} is an illegal character", c))),
            }
        })
    })
}

impl Seat {
    fn new_for_code(code: String) -> Result<Seat> {
        let i = to_id(&code)?;
        match i {
            id if id <= 1023 => Ok(Seat {
                id,
                code,
            }),
            id => Err(Error::new(format!("id {} is too high", id))),
        }
    }
}

impl PartialOrd for Seat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Seat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

fn read_lines<P: AsRef<path::Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_seats(filename: &str) -> Result<Vec<Seat>> {
    read_lines(filename)?
        .map(|res| match res {
            Ok(code) => Seat::new_for_code(code),
            Err(e) => Err(Error::new(format!("bad line: {}", e))),
        })
        .fold(Ok(Vec::new()), |acc, res| {
            acc.and_then(|mut v| {
                res.map(|seat| {
                    v.push(seat);
                    v
                })
            })
        })
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let mut seats = read_seats(&args[1])?;

        seats.sort();
        let lowest = seats.first().unwrap().id;
        let highest = seats.last().unwrap().id;
        let mine = (lowest..=highest)
            .find(|id| seats[(id - lowest) as usize].id != *id)
            .unwrap();

        println!("The lowest seat id is {}", lowest);
        println!("The highest seat id is {}", highest);
        println!("My seat id is {}", mine);
        Ok(())
    } else {
        panic!("{}", Error::new("filename argument required".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_seat_from_code {
        ($code:literal, $row:literal, $column:literal, $id:literal) => {
            let seat = Seat::new_for_code($code.to_string())?;
            assert_eq!($id, seat.id, "wrong id");
        };
    }

    #[test]
    fn converts_code_to_the_correct_seat() -> Result<()> {
        // max seat
        test_seat_from_code!("BBBBBBBRRR", 127, 7, 1023);

        // min seat
        test_seat_from_code!("FFFFFFFLLL", 0, 0, 0);

        // seats from question
        test_seat_from_code!("FBFBBFFRLR", 44, 5, 357);
        test_seat_from_code!("BFFFBBFRRR", 70, 7, 567);
        test_seat_from_code!("FFFBBBFRRR", 14, 7, 119);
        test_seat_from_code!("BBFFBBFRLL", 102, 4, 820);
        Ok(())
    }
}
