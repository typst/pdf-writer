use super::*;
use crate::buf::Buf;

/// Way the function is defined in.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum FunctionType {
    /// A function that is derived from a set of sampled data.
    Sampled,
    /// A exponential function.
    Exponential,
    /// A composite function made up of multiple other functions.
    Stitching,
    /// A postscript function.
    PostScript,
}

impl FunctionType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Sampled => 0,
            Self::Exponential => 2,
            Self::Stitching => 3,
            Self::PostScript => 4,
        }
    }
}

macro_rules! common_func_methods {
    () => {
        /// Write the `/Domain` attribute to set where the function is defined.
        /// Required.
        pub fn domain(&mut self, domain: impl IntoIterator<Item = f32>) -> &mut Self {
            self.insert(Name(b"Domain")).array().items(domain);
            self
        }

        /// Write the `/Range` attribute.
        ///
        /// Required for sampled and PostScript functions.
        pub fn range(&mut self, range: impl IntoIterator<Item = f32>) -> &mut Self {
            self.insert(Name(b"Range")).array().items(range);
            self
        }
    };
}

/// Writer for a _sampled function stream_. PDF 1.2+.
///
/// This struct is created by [`Chunk::sampled_function`].
pub struct SampledFunction<'a> {
    stream: Stream<'a>,
}

impl<'a> SampledFunction<'a> {
    /// Create a new sampled function writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"FunctionType"), FunctionType::Sampled.to_int());
        Self { stream }
    }

    common_func_methods!();

    /// Write the `/Size` attribute.
    ///
    /// Sets the number of input samples per dimension. Required.
    pub fn size(&mut self, size: impl IntoIterator<Item = i32>) -> &mut Self {
        self.insert(Name(b"Size")).array().items(size);
        self
    }

    /// Write the `/BitsPerSample` attribute.
    ///
    /// Sets the number of bits per input sample. Required.
    pub fn bits_per_sample(&mut self, bits: i32) -> &mut Self {
        self.pair(Name(b"BitsPerSample"), bits);
        self
    }

    /// Write the `/Order` attribute.
    ///
    /// Choose the implementation kind.
    pub fn order(&mut self, order: InterpolationOrder) -> &mut Self {
        self.pair(Name(b"Order"), order.to_int());
        self
    }

    /// Write the `/Encode` attribute.
    ///
    /// For each sample, define how the input is mapped to the domain range.
    pub fn encode(&mut self, encode: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Encode")).array().items(encode);
        self
    }

    /// Write the `/Decode` attribute.
    ///
    /// For each sample, define how the output is mapped to the output range.
    pub fn decode(&mut self, decode: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Decode")).array().items(decode);
        self
    }
}

deref!('a, SampledFunction<'a> => Stream<'a>, stream);

/// How to interpolate between the samples in a function of the
/// sampled type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InterpolationOrder {
    /// Linear spline interpolation.
    Linear,
    /// Cubic spline interpolation.
    Cubic,
}

impl InterpolationOrder {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Linear => 1,
            Self::Cubic => 3,
        }
    }
}

/// Writer for an _exponential function dictionary_. PDF 1.3+.
///
/// The function result is `y_i = C0_i + x^N * (C1_i - C0_i)` where `i` is the
/// current dimension.
///
/// This struct is created by [`Chunk::exponential_function`] and
/// [`writers::Separation::tint_exponential`].
pub struct ExponentialFunction<'a> {
    dict: Dict<'a>,
}

writer!(ExponentialFunction: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"FunctionType"), FunctionType::Exponential.to_int());
    Self { dict }
});

impl<'a> ExponentialFunction<'a> {
    common_func_methods!();

    /// Write the `/C0` array.
    ///
    /// Function result when input is zero. Default is `0.0`.
    pub fn c0(&mut self, c0: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"C0")).array().items(c0);
        self
    }

    /// Write the `/C1` array.
    ///
    /// Function result when input is one. Default is `1.0`.
    pub fn c1(&mut self, c1: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"C1")).array().items(c1);
        self
    }

    /// Write the `/N` attribute.
    ///
    /// The interpolation exponent. Required.
    pub fn n(&mut self, n: f32) -> &mut Self {
        self.pair(Name(b"N"), n);
        self
    }
}

deref!('a, ExponentialFunction<'a> => Dict<'a>, dict);

/// Writer for a _stitching function dictionary_. PDF 1.3+.
///
/// The function result is `y_i = C0_i + x^N * (C1_i - C0_i)` where `i` is the
/// current dimension.
///
/// This struct is created by [`Chunk::stitching_function`] and
/// [`writers::Separation::tint_stitching`].
pub struct StitchingFunction<'a> {
    dict: Dict<'a>,
}

writer!(StitchingFunction: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"FunctionType"), FunctionType::Stitching.to_int());
    Self { dict }
});

impl<'a> StitchingFunction<'a> {
    common_func_methods!();

    /// Write the `/Functions` array.
    ///
    /// The functions to be stitched. Required.
    pub fn functions(&mut self, functions: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Functions")).array().items(functions);
        self
    }

    /// Write the `/Bounds` array.
    ///
    /// The boundaries of the intervals that each function is called in. The
    /// array has one less entry than there are stiched functions. Required.
    pub fn bounds(&mut self, bounds: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Bounds")).array().items(bounds);
        self
    }

    /// Write the `/Encode` array.
    ///
    /// Pair of values for each function that maps the stitching domain subsets
    /// to the function domain. Required.
    pub fn encode(&mut self, encode: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Encode")).array().items(encode);
        self
    }
}

deref!('a, StitchingFunction<'a> => Dict<'a>, dict);

/// Writer for a _PostScript function stream_. PDF 1.3+.
///
/// This struct is created by [`Chunk::post_script_function`].
pub struct PostScriptFunction<'a> {
    stream: Stream<'a>,
}

impl<'a> PostScriptFunction<'a> {
    /// Create a new postscript function writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"FunctionType"), FunctionType::PostScript.to_int());
        Self { stream }
    }

    common_func_methods!();
}

deref!('a, PostScriptFunction<'a> => Stream<'a>, stream);

/// PostScript operators for use in Type 4 functions.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PostScriptOp<'a> {
    /// Push a real number.
    Real(f32),
    /// Push an integer number.
    Integer(i32),

    /// Absolute value. One number argument.
    Abs,
    /// Addition. Two number arguments.
    Add,
    /// Arc tangent. One number argument.
    Atan,
    /// Round up to the nearest integer. One number argument.
    Ceiling,
    /// Cosine. One number argument.
    Cos,
    /// Convert to integer. One real number argument.
    Cvi,
    /// Convert to real. One integer argument.
    Cvr,
    /// Divide. Two number arguments.
    Div,
    /// Raise the base to the exponent. Two number arguments.
    Exp,
    /// Round down to the nearest integer. One number argument.
    Floor,
    /// Integer division. Two integer arguments.
    Idiv,
    /// Natural logarithm. One number argument.
    Ln,
    /// Logarithm base 10. One number argument.
    Log,
    /// Modulo. Two integer arguments.
    Mod,
    /// Multiply. Two number arguments.
    Mul,
    /// Negate. One number argument.
    Neg,
    /// Round to the nearest integer. One number argument.
    Round,
    /// Sine. One number argument.
    Sin,
    /// Square root. One number argument.
    Sqrt,
    /// Subtract. Two number arguments.
    Sub,
    /// Remove fractional part. One number argument.
    Truncate,

    /// Logical bitwise And. Two integer or boolean arguments.
    And,
    /// Bitwise shift left. Negative shifts possible. Two integer arguments.
    Bitshift,
    /// Equals. Any two arguments of the same type.
    Eq,
    /// Constant false.
    False,
    /// Greater than or equal. Two number arguments.
    Ge,
    /// Greater than. Two number arguments.
    Gt,
    /// Less than or equal. Two number arguments.
    Le,
    /// Less than. Two number arguments.
    Lt,
    /// Not equals. Any two arguments of the same type.
    Ne,
    /// Bitwise logical not. One integer or boolean argument.
    Not,
    /// Bitwise logical or. Two integer or boolean arguments.
    Or,
    /// Constant true.
    True,
    /// Bitwise logical exclusive or. Two integer or boolean arguments.
    Xor,

    /// Conditional. Runs if boolean argument is true.
    If(&'a [Self]),
    /// Conditional. Decides which branch to run depending on boolean argument.
    IfElse(&'a [Self], &'a [Self]),

    /// Copy the top elements. One integer argument.
    Copy,
    /// Duplicate the top element.
    Dup,
    /// Exchange the two top elements.
    Exch,
    /// Duplicate any element. One integer argument.
    Index,
    /// Discard the top element.
    Pop,
    /// Roll `n` elements up `j` times. Two integer arguments.
    Roll,
}

impl<'a> PostScriptOp<'a> {
    /// Encode a slice of operations into a byte stream.
    pub fn encode(ops: &[Self]) -> Buf {
        let mut buf = Buf::new();
        Self::write_slice(ops, &mut buf);
        buf
    }

    fn write_slice(ops: &[Self], buf: &mut Buf) {
        buf.push(b'{');
        if ops.len() > 1 {
            buf.push(b'\n');
        }
        for op in ops {
            op.write(buf);
            buf.push(b'\n');
        }
        if ops.len() == 1 {
            buf.pop();
        }
        buf.push(b'}');
    }

    fn write(&self, buf: &mut Buf) {
        match *self {
            Self::Real(r) => buf.push_decimal(r),
            Self::Integer(i) => buf.push_val(i),
            Self::If(ops) => {
                Self::write_slice(ops, buf);
                buf.push(b'\n');
                buf.extend_slice(self.operator());
            }
            Self::IfElse(ops1, ops2) => {
                Self::write_slice(ops1, buf);
                buf.push(b'\n');
                Self::write_slice(ops2, buf);
                buf.push(b'\n');
                buf.extend_slice(self.operator());
            }
            _ => buf.extend_slice(self.operator()),
        }
    }

    fn operator(&self) -> &'static [u8] {
        match self {
            Self::Real(_) | Self::Integer(_) => b"",
            Self::Abs => b"abs",
            Self::Add => b"add",
            Self::Atan => b"atan",
            Self::Ceiling => b"ceiling",
            Self::Cos => b"cos",
            Self::Cvi => b"cvi",
            Self::Cvr => b"cvr",
            Self::Div => b"div",
            Self::Exp => b"exp",
            Self::Floor => b"floor",
            Self::Idiv => b"idiv",
            Self::Ln => b"ln",
            Self::Log => b"log",
            Self::Mod => b"mod",
            Self::Mul => b"mul",
            Self::Neg => b"neg",
            Self::Round => b"round",
            Self::Sin => b"sin",
            Self::Sqrt => b"sqrt",
            Self::Sub => b"sub",
            Self::Truncate => b"truncate",
            Self::And => b"and",
            Self::Bitshift => b"bitshift",
            Self::Eq => b"eq",
            Self::False => b"false",
            Self::Ge => b"ge",
            Self::Gt => b"gt",
            Self::Le => b"le",
            Self::Lt => b"lt",
            Self::Ne => b"ne",
            Self::Not => b"not",
            Self::Or => b"or",
            Self::True => b"true",
            Self::Xor => b"xor",
            Self::If(_) => b"if",
            Self::IfElse(_, _) => b"ifelse",
            Self::Copy => b"copy",
            Self::Dup => b"dup",
            Self::Exch => b"exch",
            Self::Index => b"index",
            Self::Pop => b"pop",
            Self::Roll => b"roll",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_script_encoding() {
        use PostScriptOp::*;

        let ops = [
            Real(3.0),
            Real(2.0),
            Mul,
            Exch,
            Dup,
            Real(0.0),
            Ge,
            IfElse(&[Real(1.0), Add], &[Neg]),
            Add,
        ];

        assert_eq!(
            PostScriptOp::encode(&ops).to_bytes(),
            b"{\n3.0\n2.0\nmul\nexch\ndup\n0.0\nge\n{\n1.0\nadd\n}\n{neg}\nifelse\nadd\n}"
        );
    }
}
