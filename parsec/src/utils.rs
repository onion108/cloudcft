use std::fmt::Debug;
use std::marker::PhantomData;
use crate::prelude::*;

pub struct BoxedParser<'a, I: 'a + Clone, O: 'a + Clone> {
  parser: Box<dyn Parser<'a, I, O> + 'a>,
}

impl<'a, I: 'a + Clone, O: 'a + Clone> Parser<'a, I, O> for BoxedParser<'a, I, O> {
  fn parse(&self, input: I) -> ParseResult<'a, I, O> {
    self.parser.parse(input)
  }
}
impl<'a, I: 'a + Clone, O: 'a + Clone> Parser<'a, I, O> for &BoxedParser<'a, I, O> {
  fn parse(&self, input: I) -> ParseResult<'a, I, O> {
    self.parser.parse(input)
  }
}
impl<'a, I: 'a + Clone, O: 'a + Clone> BoxedParser<'a, I, O> {
  pub fn new(parser: impl Parser<'a, I, O> + 'a) -> Self {
    BoxedParser {
      parser: Box::new(parser),
    }
  }
}

/// Converts `ParseResult<I, A>` to `ParseResult<I, B>` and returns you a parser to do that.
pub struct Map<'a, I: 'a + Clone, A: 'a + Clone, B: 'a + Clone> {
  parser: BoxedParser<'a, I, A>,
  map_fn: Box<dyn Fn(A) -> B + 'a>,
}
impl<'a, I: 'a + Clone, A: 'a + Clone, B: 'a + Clone> Parser<'a, I, B> for Map<'a, I, A, B> {
  fn parse(&self, input: I) -> ParseResult<'a, I, B> {
    self.parser.parse(input).map(|r| r.map(|a| (self.map_fn)(a)))
  }
}

/// Refer to `struct Map`.
pub fn map<'a, I: 'a + Clone, A: 'a + Clone, B: 'a + Clone>(
  parser: impl Parser<'a, I, A> + 'a,
  map_fn: impl Fn(A) -> B + 'a
) -> Map<'a, I, A, B> {
  Map {
    parser: BoxedParser::new(parser),
    map_fn: Box::new(map_fn),
  }
}

pub struct Or<'a, I: 'a + Clone, O: 'a + Clone> {
  parser_a: BoxedParser<'a, I, O>,
  parser_b: BoxedParser<'a, I, O>,
}
impl<'a, I: 'a + Clone, O: 'a + Clone> Parser<'a, I, O> for Or<'a, I, O> {
  fn parse(&self, input: I) -> ParseResult<'a, I, O> {
    self.parser_a.parse(input.clone()).or_else(|_| {
      self.parser_b.parse(input.clone())
    })
  }
}
pub fn or<'a, I: 'a + Clone, O: 'a + Clone>(
  parser_a: impl Parser<'a, I, O> + 'a,
  parser_b: impl Parser<'a, I, O> + 'a,
) -> Or<'a, I, O> {
  Or {
    parser_a: BoxedParser::new(parser_a),
    parser_b: BoxedParser::new(parser_b),
  }
}

pub struct OrDefault<'a, I: 'a + Clone, O: 'a + Clone> {
  parser: Box<dyn Parser<'a, I, O> + 'a>,
  default: O,
}
impl<'a, I: 'a + Clone, O: 'a + Clone> Parser<'a, I, O> for OrDefault<'a, I, O> {
  fn parse(&self, input: I) -> ParseResult<'a, I, O> {
    self.parser
      .parse(input.clone())
      .or_else(|_| Ok(ResultData::new(input, self.default.clone())))
  }
}
pub fn or_default<'a, I, O>(
  parser: impl Parser<'a, I, O> + 'a,
  default: O
) -> OrDefault<'a, I, O>
where
  I: 'a + Clone,
  O: 'a + Clone
{
  OrDefault {
    parser: Box::new(parser),
    default,
  }
}

pub struct And<'a, I: 'a, A: 'a, B: 'a> {
  parser_a: Box<dyn Parser<'a, I, A> + 'a>,
  parser_b: Box<dyn Parser<'a, I, B> + 'a>,
}

impl<'a, I: 'a + Clone + Debug, A: 'a + Clone + Debug, B: 'a + Clone> Parser<'a, I, (A, B)> for And<'a, I, A, B> {
  fn parse(&self, input: I) -> ParseResult<'a, I, (A, B)> {
    self.parser_a.parse(input).and_then(|a| {
      self.parser_b.parse(a.remain).map(|b| {
        ResultData::new(
          b.remain,
          (a.output, b.output)
        )
      })
    })
  }
}

pub fn and<'a, I: Clone, A: Clone, B: Clone>(
  parser_a: impl Parser<'a, I, A> + 'a,
  parser_b: impl Parser<'a, I, B> + 'a
) -> And<'a, I, A, B> {
  And {
    parser_a: Box::new(parser_a),
    parser_b: Box::new(parser_b),
  }
}

pub struct Only<'a, I: 'a + Clone, D: 'a + Clone> {
  return_value: D,
  _marker: PhantomData<&'a I>
}
impl<'a, I: 'a + Clone, D: 'a + Clone> Parser<'a, I, D> for Only<'a, I, D> {
  fn parse(&self, input: I) -> ParseResult<'a, I, D> {
    Ok(ResultData::new(input, self.return_value.clone()))
  }
}
impl<'a, I: 'a + Clone, D: 'a + Clone> Only<'a, I, D> {
  pub fn new(return_value: D) -> Self {
    Only {
      return_value,
      _marker: PhantomData,
    }
  }
}
pub fn only<'a, I: 'a + Clone, D: 'a + Clone>(return_value: D) -> Only<'a, I, D> {
  Only::new(return_value)
}

pub fn some<'a, I: 'a + Clone, O: 'a + Clone>(parser: impl Parser<'a, I, O>) -> impl Parser<'a, I, Vec<O>> {
  move |input: I| -> ParseResult<'a, I, Vec<O>> {
    let mut results: Vec<O> = vec![];
    let mut next_input: I = input;
    while let Ok(result) = parser.parse(next_input.clone()) {
      next_input = result.remain;
      results.push(result.output);
    }
    Ok(ResultData::new(next_input, results))
  }
}

pub fn more<'a, I: 'a+ Clone, O: 'a + Clone>(parser: impl Parser<'a, I, O> + 'a) -> impl Parser<'a, I, Vec<O>> {
  filter(some(parser), |result| result.output.is_empty())
}

pub fn filter<'a, I: 'a + Clone, O: 'a + Clone>(
  parser: impl Parser<'a, I, O> + 'a,
  condition: impl Fn(ResultData<'a, I, O>) -> bool)  -> impl Parser<'a, I, O> {
  move |input: I| -> ParseResult<'a, I, O> {
    parser.parse(input)
      .ok()
      .filter(|a| condition(a.to_owned()))
      .ok_or_else(|| "".to_string())
  }
}

pub fn to_option<'a, I: 'a + Clone, O: 'a + Clone>(parser: impl Parser<'a, I, O>) -> impl Parser<'a, I, Option<O>> {
  move |input: I| -> ParseResult<'a, I, Option<O>> {
    let result = parser.parse(input.clone());
    result.to_owned()
      .map(|result_data| {
        ResultData::new(
          result_data.remain,
          result.ok().map(|result_data| {
            result_data.output
        }))
      })
      .or_else(|_| {
        Ok(ResultData::new(input, None))
      })
  }
}

pub trait ParserExt<'a, I: 'a + Clone, O: 'a + Clone>: Parser<'a, I, O>
where Self: Sized + 'a
{
  fn map<B: 'a + Clone>(self, map_fn: impl Fn(O) -> B + 'a) -> Map<'a, I, O, B> {
    map(self, map_fn)
  }

  fn and<B: 'a + Clone>(self, parser_b: impl Parser<'a, I, B> + 'a) -> And<'a, I, O, B> {
    and(self, parser_b)
  }

  fn or(self, another: impl Parser<'a, I, O> + 'a) -> Or<'a, I, O> {
    or(self, another)
  }
}

impl<'a, I: 'a + Clone, O: 'a + Clone, P> ParserExt<'a, I, O> for P
where P: Parser<'a, I, O> + 'a
{}

