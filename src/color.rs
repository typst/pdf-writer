use super::*;

/// CIE XYZ coordinates of the D65 noon daylight white.
const CIE_D65: [f32; 3] = [0.9505, 1.0, 1.0888];

/// CIE XYZ coordinates of the D50 horizon light white.
const CIE_D50: [f32; 3] = [0.9642, 1.0, 0.8251];

/// CIE XYZ coordinates of the E equal radiator white.
const CIE_E: [f32; 3] = [1.000, 1.000, 1.000];

/// CIE XYZ coordinates of the C north sky daylight white.
const CIE_C: [f32; 3] = [0.9807, 1.0000, 1.1822];

/// The type of a color space.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(unused)]
#[allow(missing_docs)]
pub enum ColorSpaceType {
    CalGray,
    CalRgb,
    Lab,
    IccBased,
    DeviceRgb,
    DeviceCmyk,
    DeviceGray,
    Indexed,
    Pattern,
    Separation,
    DeviceN,
}

impl ColorSpaceType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::CalRgb => Name(b"CalRGB"),
            Self::CalGray => Name(b"CalGray"),
            Self::Lab => Name(b"Lab"),
            Self::IccBased => Name(b"ICCBased"),
            Self::DeviceRgb => Name(b"DeviceRGB"),
            Self::DeviceCmyk => Name(b"DeviceCMYK"),
            Self::DeviceGray => Name(b"DeviceGray"),
            Self::Separation => Name(b"Separation"),
            Self::DeviceN => Name(b"DeviceN"),
            Self::Indexed => Name(b"Indexed"),
            Self::Pattern => Name(b"Pattern"),
        }
    }

    pub fn is_device(self) -> bool {
        match self {
            Self::CalRgb | Self::CalGray | Self::Lab | Self::IccBased => false,
            Self::DeviceRgb | Self::DeviceCmyk | Self::DeviceGray | Self::Indexed => true,
            Self::Separation | Self::DeviceN | Self::Pattern => false,
        }
    }
}

/// Writer for a _color space_.
///
/// This struct is created by [`PdfWriter::color_space`],
/// [`Shading::color_space`], [`ImageXObject::color_space`],
/// [`Separation::alternate_color_space`] and [`Group::color_space`].
pub struct ColorSpace<'a> {
    obj: Obj<'a>,
}

writer!(ColorSpace: |obj| Self { obj });

/// CIE-based color spaces.
impl ColorSpace<'_> {
    /// Write a `CalRGB` color space.
    pub fn cal_rgb(
        self,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<[f32; 3]>,
        matrix: Option<[f32; 9]>,
    ) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::CalRgb.to_name());

        let mut dict = array.push().dict();
        dict.insert(Name(b"WhitePoint")).array().items(white_point);

        if let Some(black_point) = black_point {
            dict.insert(Name(b"BlackPoint")).array().items(black_point);
        }

        if let Some(gamma) = gamma {
            dict.insert(Name(b"Gamma")).array().items(gamma);
        }

        if let Some(matrix) = matrix {
            dict.insert(Name(b"Matrix")).array().items(matrix);
        }
    }

    /// Write a `CalGray` color space.
    pub fn cal_gray(
        self,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<f32>,
    ) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::CalGray.to_name());

        let mut dict = array.push().dict();
        dict.insert(Name(b"WhitePoint")).array().items(white_point);

        if let Some(black_point) = black_point {
            dict.insert(Name(b"BlackPoint")).array().items(black_point);
        }

        if let Some(gamma) = gamma {
            dict.pair(Name(b"Gamma"), gamma);
        }
    }

    /// Write a `Lab` color space.
    pub fn lab(
        self,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        range: Option<[f32; 4]>,
    ) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::Lab.to_name());

        let mut dict = array.push().dict();
        dict.insert(Name(b"WhitePoint")).array().items(white_point);

        if let Some(black_point) = black_point {
            dict.insert(Name(b"BlackPoint")).array().items(black_point);
        }

        if let Some(range) = range {
            dict.insert(Name(b"Range")).array().items(range);
        }
    }

    /// Write an `ICCBased` color space.
    ///
    /// The `stream` argument is an indirect reference to an [ICC
    /// profile](IccProfile) stream.
    pub fn icc_based(self, stream: Ref) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::IccBased.to_name());
        array.item(stream);
    }
}

/// Writer for an _ICC profile stream_.
///
/// This struct is created by [`PdfWriter::icc_profile`].
pub struct IccProfile<'a> {
    stream: Stream<'a>,
}

impl<'a> IccProfile<'a> {
    /// Create a new ICC profile stream writer
    pub(crate) fn start(stream: Stream<'a>) -> Self {
        Self { stream }
    }

    /// Write the `/N` attribute. Required.
    ///
    /// The number of components in the color space.
    /// Shall be 1, 3, or 4.
    pub fn n(&mut self, n: i32) -> &mut Self {
        assert!(
            n == 1 || n == 3 || n == 4,
            "n must be 1, 3, or 4, but is {}",
            n
        );
        self.pair(Name(b"N"), n);
        self
    }

    /// Write the `/Alternate` attribute with a color space.
    ///
    /// The alternate color space to use when the ICC profile is not
    /// supported. Must be a color space with the same number of
    /// components as the ICC profile. Pattern color spaces are not
    /// allowed.
    pub fn alternate(&mut self) -> ColorSpace<'_> {
        ColorSpace::start(self.insert(Name(b"Alternate")))
    }

    /// Write the `/Alternate` attribute with a name.
    ///
    /// The alternate color space referenced by name must be registered in the
    /// current [resource dictionary.](crate::writers::Resources)
    pub fn alternate_name(&mut self, name: Name<'_>) -> &mut Self {
        self.pair(Name(b"Alternate"), name);
        self
    }

    /// Write the `/Range` attribute.
    ///
    /// Specifies the permissible range of values for each component. The array
    /// shall contain 2 × `N` numbers, where [`N`](Self::n) is the number of
    /// components in the color space. The array is organized in pairs, where
    /// the first value shall be the minimum value and the second shall be the
    /// maximum value.
    pub fn range(&mut self, range: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Range")).array().typed().items(range);
        self
    }

    /// Write the `/Metadata` attribute.
    ///
    /// A reference to a [stream containing metadata](crate::writers::Metadata)
    /// for the ICC profile.
    pub fn metadata(&mut self, metadata: Ref) -> &mut Self {
        self.pair(Name(b"Metadata"), metadata);
        self
    }
}

deref!('a, IccProfile<'a> => Stream<'a>, stream);

/// Common calibrated color spaces.
impl ColorSpace<'_> {
    /// Write a `CalRGB` color space approximating sRGB.
    ///
    /// Use an ICC profile for more accurate results.
    pub fn srgb(self) {
        self.cal_rgb(
            CIE_D65,
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.4124, 0.2126, 0.0193, 0.3576, 0.715, 0.1192, 0.1805, 0.0722, 0.9505,
            ]),
        )
    }

    /// Write a `CalRGB` color space approximating Adobe RGB.
    ///
    /// Use an ICC profile for more accurate results.
    pub fn adobe_rgb(self) {
        self.cal_rgb(
            CIE_D65,
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.57667, 0.29734, 0.02703, 0.18556, 0.62736, 0.07069, 0.18823, 0.07529,
                0.99134,
            ]),
        )
    }

    /// Write a `CalRGB` color space approximating Display P3.
    ///
    /// Use an ICC profile for more accurate results.
    pub fn display_p3(self) {
        self.cal_rgb(
            CIE_D65,
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.48657, 0.2297, 0.0, 0.26567, 0.69174, 0.04511, 0.19822, 0.07929,
                1.04394,
            ]),
        )
    }

    /// Write a `CalRGB` color space approximating ProPhoto.
    ///
    /// Use an ICC profile for more accurate results.
    pub fn pro_photo(self) {
        self.cal_rgb(
            CIE_D50,
            None,
            Some([1.8, 1.8, 1.8]),
            Some([
                0.7976749, 0.2880402, 0.0, 0.1351917, 0.7118741, 0.0, 0.0313534,
                0.0000857, 0.8252100,
            ]),
        )
    }

    /// Write a `CalRGB` color space for ECI RGB v1.
    pub fn eci_rgb(self) {
        self.cal_rgb(
            CIE_D50,
            None,
            Some([1.8, 1.8, 1.8]),
            Some([
                0.6502043, 0.3202499, 0.0, 0.1780774, 0.6020711, 0.0678390, 0.1359384,
                0.0776791, 0.7573710,
            ]),
        )
    }

    /// Write a `CalRGB` color space for NTSC RGB.
    pub fn ntsc(self) {
        self.cal_rgb(
            CIE_C,
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.6068909, 0.2989164, 0.0, 0.1735011, 0.5865990, 0.0660957, 0.2003480,
                0.1144845, 1.1162243,
            ]),
        )
    }

    /// Write a `CalRGB` color space for PAL/SECAM RGB.
    pub fn pal(self) {
        self.cal_rgb(
            CIE_D65,
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.4306190, 0.2220379, 0.0201853, 0.3415419, 0.7066384, 0.1295504,
                0.1783091, 0.0713236, 0.9390944,
            ]),
        )
    }

    /// Write a `CalGray` color space for CIE D65 at a 2.2 gamma, equivalent to
    /// sRGB, Adobe RGB, Display P3, PAL, ...
    pub fn d65_gray(self) {
        self.cal_gray(CIE_D65, None, Some(2.2))
    }

    /// Write a `CalGray` color space for CIE D50 (horizon light). Set a 1.8
    /// gamma for ProPhoto or ECI RGB equivalency, 2.2 is another common value.
    pub fn d50_gray(self, gamma: Option<f32>) {
        self.cal_gray(CIE_D50, None, gamma)
    }

    /// Write a `CalGray` color space for CIE C (north sky daylight) at 2.2
    /// gamma, equivalent to NTSC.
    pub fn c_gray(self) {
        self.cal_gray(CIE_C, None, Some(2.2))
    }

    /// Write a `CalGray` color space for CIE E (equal emission). Common gamma
    /// values include 1.8 or 2.2.
    pub fn e_gray(self, gamma: Option<f32>) {
        self.cal_gray(CIE_E, None, gamma)
    }
}

/// Device color spaces.
impl ColorSpace<'_> {
    /// Write a `DeviceRGB` color space.
    pub fn device_rgb(self) {
        self.obj.primitive(ColorSpaceType::DeviceRgb.to_name());
    }

    /// Write a `DeviceCMYK` color space.
    pub fn device_cmyk(self) {
        self.obj.primitive(ColorSpaceType::DeviceCmyk.to_name());
    }

    /// Write a `DeviceGray` color space.
    pub fn device_gray(self) {
        self.obj.primitive(ColorSpaceType::DeviceGray.to_name());
    }
}

/// Special color spaces.
impl<'a> ColorSpace<'a> {
    /// Start writing a `Separation` color space. PDF 1.2+.
    ///
    /// The `color_name` argument is the name of the colorant that will be
    /// used by the printer.
    pub fn separation(self, color_name: Name) -> Separation<'a> {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::Separation.to_name());
        array.item(color_name);
        Separation::start(array)
    }

    /// Write a `DeviceN` color space. PDF 1.3+.
    ///
    /// The `names` argument contains the N names of the color components and
    /// the respective colorants. The `alternate_space` argument describes an
    /// alternate color space to use if the color names are unknown. The `tint`
    /// argument is an indirect reference to a function that maps from an n-
    /// dimensional values with components between 0 and 1 to a color in the
    /// alternate color space.
    pub fn device_n<'n>(
        self,
        names: impl IntoIterator<Item = Name<'n>>,
        alternate_space: Name,
        tint: Ref,
    ) -> DeviceN<'a> {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::DeviceN.to_name());
        array.push().array().items(names);
        array.item(alternate_space);
        array.item(tint);
        DeviceN::start(array)
    }

    /// Write an `Indexed` color space. PDF 1.2+.
    ///
    /// The length of the lookup slice must be the product of the dimensions of
    /// the base color space and (`hival + 1`) and `hival` shall be at most 255.
    pub fn indexed(self, base: Name, hival: i32, lookup: &[u8]) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::Indexed.to_name());
        array.item(base);
        array.item(hival);
        array.item(Str(lookup));
    }

    /// Write a `Pattern` color space for uncolored patterns. PDF 1.2+.
    ///
    /// The `base` attribute is the color space in which the pattern's
    /// [tint](Content::set_stroke_pattern) color is specified upon use.
    pub fn pattern(self, base: Name) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::Pattern.to_name());
        array.item(base);
    }
}

/// Type of pattern.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum PatternType {
    /// A tiling pattern.
    Tiling,
    /// A shading pattern.
    Shading,
}

impl PatternType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Tiling => 1,
            Self::Shading => 2,
        }
    }
}

/// Writer for a _separation dictionary_. PDF 1.2+.
///
/// First, one of the `alternate_...` methods must be called to specify the
/// alternate color space. Then, one of the `tint_...` methods must be called
/// to specify the tint transform function. If the tint transform function is
/// called before the alternate color space, the function panics. If multiple
/// alternate color space functions are called, the function panics.
/// This struct is created by [`ColorSpace::separation`].
pub struct Separation<'a> {
    array: Array<'a>,
    has_alternate: bool,
}

impl<'a> Separation<'a> {
    /// Start the wrapper.
    pub(crate) fn start(array: Array<'a>) -> Self {
        Self { array, has_alternate: false }
    }

    /// Write the `alternateSpace` element as a name. The argument must be a
    /// Device color space or else the function panics.
    pub fn alternate_device(&mut self, device_space: ColorSpaceType) -> &mut Self {
        if self.has_alternate {
            panic!("alternateSpace already specified");
        }
        if !device_space.is_device() {
            panic!("alternateSpace must be a Device color space");
        }
        self.array.item(device_space.to_name());
        self.has_alternate = true;
        self
    }

    /// Start writing the `alternateSpace` element as a color space array. The
    /// color space must not be another `Pattern`, `Separation`, or `DeviceN`
    /// color space.
    pub fn alternate_color_space(&mut self) -> ColorSpace<'_> {
        if self.has_alternate {
            panic!("alternateSpace already specified");
        }
        self.has_alternate = true;
        ColorSpace::start(self.array.push())
    }

    /// Write the `alternateSpace` element as an indirect reference. The color
    /// space must not be another `Pattern`, `Separation`, or `DeviceN` color
    /// space.
    pub fn alternate_color_space_ref(&mut self, id: Ref) -> &mut Self {
        if self.has_alternate {
            panic!("alternateSpace already specified");
        }
        self.array.item(id);
        self.has_alternate = true;
        self
    }

    /// Write the `tintTransform` element as an indirect reference to a
    /// function. The function must take a single number as input and produce a
    /// color in the alternate color space as output. This must be used if a
    /// stream function like [`SampledFunction`] or [`PostScriptFunction`] is
    /// used.
    pub fn tint_ref(&mut self, id: Ref) -> &mut Self {
        if !self.has_alternate {
            panic!("alternateSpace must be specified before tintTransform");
        }
        self.array.item(id);
        self
    }

    /// Start writing the `tintTransform` element as an exponential
    /// interpolation function.
    pub fn tint_exponential(&mut self) -> ExponentialFunction<'_> {
        if !self.has_alternate {
            panic!("alternateSpace must be specified before tintTransform");
        }
        ExponentialFunction::start(self.array.push())
    }

    /// Start writing the `tintTransform` element as a stitching function.
    pub fn tint_stitching(&mut self) -> StitchingFunction<'_> {
        if !self.has_alternate {
            panic!("alternateSpace must be specified before tintTransform");
        }
        StitchingFunction::start(self.array.push())
    }
}

/// Writer for a _DeviceN color space array with attributes_. PDF 1.6+.
///
/// This struct is created by [`ColorSpace::device_n`].
pub struct DeviceN<'a> {
    array: Array<'a>,
}

impl<'a> DeviceN<'a> {
    /// Start the wrapper.
    pub(crate) fn start(array: Array<'a>) -> Self {
        Self { array }
    }

    /// Start writing the `attrs` dictionary. PDF 1.6+.
    pub fn attrs(&mut self) -> DeviceNAttrs<'_> {
        DeviceNAttrs::start(self.array.push())
    }
}

/// Writer for a _DeviceN attributes dictionary_. PDF 1.6+.
///
/// This struct is created by [`DeviceN::attrs`].
pub struct DeviceNAttrs<'a> {
    dict: Dict<'a>,
}

writer!(DeviceNAttrs: |obj| Self { dict: obj.dict() });

impl DeviceNAttrs<'_> {
    /// Write the `/Subtype` attribute.
    pub fn subtype(&mut self, subtype: DeviceNSubtype) -> &mut Self {
        self.dict.pair(Name(b"Subtype"), subtype.to_name());
        self
    }

    /// Start writing the `/Colorants` dictionary. Its keys are the colorant
    /// names and its values are separation color space arrays.
    ///
    /// Required if the `/Subtype` attribute is `NChannel`.
    pub fn colorants(&mut self) -> Dict<'_> {
        self.dict.insert(Name(b"Colorants")).dict()
    }

    /// Start writing the `/Process` dictionary.
    ///
    /// Required if the `/Subtype` attribute is `Separation`.
    pub fn process(&mut self) -> DeviceNProcess<'_> {
        DeviceNProcess::start(self.dict.insert(Name(b"Process")))
    }

    /// Start writing the `/MixingHints` dictionary.
    pub fn mixing_hints(&mut self) -> DeviceNMixingHints<'_> {
        DeviceNMixingHints::start(self.dict.insert(Name(b"MixingHints")))
    }
}

/// Writer for a _DeviceN process dictionary_. PDF 1.6+.
///
/// This struct is created by [`DeviceNAttrs::process`].
pub struct DeviceNProcess<'a> {
    dict: Dict<'a>,
}

writer!(DeviceNProcess: |obj| Self { dict: obj.dict() });

impl DeviceNProcess<'_> {
    /// Write the `/ColorSpace` attribute with a name. Required.
    pub fn color_space(&mut self, color_space: Name) -> &mut Self {
        self.dict.pair(Name(b"ColorSpace"), color_space);
        self
    }

    /// Write the `/ColorSpace` attribute with an array. Required.
    pub fn color_space_array(&mut self) -> ColorSpace<'_> {
        ColorSpace::start(self.dict.insert(Name(b"ColorSpace")))
    }

    /// Write the `/Components` attribute. Required.
    ///
    /// Contains the names of the colorants in the order in which they appear in
    /// the color space array.
    pub fn components<'n>(
        &mut self,
        components: impl IntoIterator<Item = Name<'n>>,
    ) -> &mut Self {
        self.dict
            .insert(Name(b"Components"))
            .array()
            .typed()
            .items(components);
        self
    }
}

/// Type of n-dimensional color space.
pub enum DeviceNSubtype {
    /// A subtractive color space.
    DeviceN,
    /// An additive color space.
    NChannel,
}

impl DeviceNSubtype {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::DeviceN => Name(b"DeviceN"),
            Self::NChannel => Name(b"NChannel"),
        }
    }
}

/// Writer for a _DeviceN mixing hints dictionary_. PDF 1.6+.
///
/// This struct is created by [`DeviceNAttrs::mixing_hints`].
pub struct DeviceNMixingHints<'a> {
    dict: Dict<'a>,
}

writer!(DeviceNMixingHints: |obj| Self { dict: obj.dict() });

impl DeviceNMixingHints<'_> {
    /// Start writing the `/Solidities` dictionary.
    ///
    /// Each key in the dictionary is a colorant name and each value is a number
    /// between 0 and 1 indicating the relative solidity of the colorant.
    pub fn solidities(&mut self) -> TypedDict<'_, f32> {
        self.dict.insert(Name(b"Solidities")).dict().typed()
    }

    /// Write the `/PrintingOrder` attribute.
    ///
    /// Required if `/Solidities` is present. An array of colorant names in the
    /// order in which they should be printed.
    pub fn printing_order<'n>(
        &mut self,
        order: impl IntoIterator<Item = Name<'n>>,
    ) -> &mut Self {
        self.dict.insert(Name(b"PrintingOrder")).array().typed().items(order);
        self
    }

    /// Start writing the `/DotGain` dictionary.
    ///
    /// Each key in the dictionary is a colorant name and each value is a number
    /// between 0 and 1 indicating the dot gain of the colorant.
    pub fn dot_gain(&mut self) -> TypedDict<'_, f32> {
        self.dict.insert(Name(b"DotGain")).dict().typed()
    }
}

/// Writer for a _tiling pattern stream_.
///
/// This struct is created by [`PdfWriter::tiling_pattern`].
pub struct TilingPattern<'a> {
    stream: Stream<'a>,
}

impl<'a> TilingPattern<'a> {
    pub(crate) fn start_with_stream(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"Pattern"));
        stream.pair(Name(b"PatternType"), PatternType::Tiling.to_int());
        Self { stream }
    }

    /// Write the `/PaintType` attribute.
    ///
    /// Sets whether to use external or stream color. Required.
    pub fn paint_type(&mut self, paint_type: PaintType) -> &mut Self {
        self.stream.pair(Name(b"PaintType"), paint_type.to_int());
        self
    }

    /// Write the `/TilingType` attribute.
    ///
    /// Sets how to stretch and space the pattern. Required.
    pub fn tiling_type(&mut self, tiling_type: TilingType) -> &mut Self {
        self.stream.pair(Name(b"TilingType"), tiling_type.to_int());
        self
    }

    /// Write the `/BBox` attribute.
    ///
    /// Sets the bounding box of the pattern in the pattern's coordinate system.
    /// Required.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.stream.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/XStep` attribute.
    ///
    /// Sets the horizontal spacing between pattern cells. Required.
    ///
    /// Panics if `x_step` is zero.
    pub fn x_step(&mut self, x_step: f32) -> &mut Self {
        assert!(x_step != 0.0, "x step must not be zero");
        self.stream.pair(Name(b"XStep"), x_step);
        self
    }

    /// Write the `/YStep` attribute.
    ///
    /// Sets the vertical spacing between pattern cells. Required.
    ///
    /// Panics if `y_step` is zero.
    pub fn y_step(&mut self, y_step: f32) -> &mut Self {
        assert!(y_step != 0.0, "y step must not be zero");
        self.stream.pair(Name(b"YStep"), y_step);
        self
    }

    /// Start writing the `/Resources` dictionary.
    ///
    /// Sets the resources used by the pattern. Required.
    pub fn resources(&mut self) -> Resources<'_> {
        self.insert(Name(b"Resources")).start()
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Maps the pattern coordinate system to the parent content stream
    /// coordinates. The default is the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.stream.insert(Name(b"Matrix")).array().items(matrix);
        self
    }
}

deref!('a, TilingPattern<'a> => Stream<'a>, stream);

/// Type of paint for a tiling pattern.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PaintType {
    /// Paint the pattern with the colors specified in the stream.
    Colored,
    /// Paint the pattern with the colors active when the pattern was painted.
    Uncolored,
}

impl PaintType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Colored => 1,
            Self::Uncolored => 2,
        }
    }
}

/// How to adjust tile spacing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TilingType {
    /// Constant space between each tile, tiles may be distorted by 1px.
    ConstantSpacing,
    /// Tile size is constant, spacing between may vary by 1px.
    NoDistortion,
    /// Constant space between each tile and faster drawing, tiles may be distorted.
    FastConstantSpacing,
}

impl TilingType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::ConstantSpacing => 1,
            Self::NoDistortion => 2,
            Self::FastConstantSpacing => 3,
        }
    }
}

/// Writer for a _shading pattern dictionary_. PDF 1.3+.
///
/// This struct is created by [`PdfWriter::shading_pattern`].
pub struct ShadingPattern<'a> {
    dict: Dict<'a>,
}

writer!(ShadingPattern: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Pattern"));
    dict.pair(Name(b"PatternType"), PatternType::Shading.to_int());
    Self { dict }
});

impl<'a> ShadingPattern<'a> {
    /// Start writing the `/Shading` dictionary.
    pub fn shading(&mut self) -> Shading<'_> {
        self.dict.insert(Name(b"Shading")).start()
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Sets the matrix to use for the pattern. Defaults to the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.dict.insert(Name(b"Matrix")).array().items(matrix);
        self
    }

    /// Start writing the `/ExtGState` attribute.
    pub fn ext_graphics(&mut self) -> ExtGraphicsState<'_> {
        self.dict.insert(Name(b"ExtGState")).start()
    }
}

deref!('a, ShadingPattern<'a> => Dict< 'a>, dict);

/// Writer for a _shading dictionary_. PDF 1.3+.
///
/// This struct is created by [`PdfWriter::shading`] and
/// [`ShadingPattern::shading`].
pub struct Shading<'a> {
    dict: Dict<'a>,
}

writer!(Shading: |obj| Self { dict: obj.dict() });

impl<'a> Shading<'a> {
    /// Write the `/ShadingType` attribute.
    ///
    /// Sets the type of shading. The available and required attributes change
    /// depending on this. Required.
    pub fn shading_type(&mut self, kind: ShadingType) -> &mut Self {
        self.dict.pair(Name(b"ShadingType"), kind.to_int());
        self
    }

    /// Start writing the `/ColorSpace` attribute.
    ///
    /// Sets the color space of the shading function. May not be a `Pattern`
    /// space. Required.
    pub fn color_space(&mut self) -> ColorSpace<'_> {
        self.dict.insert(Name(b"ColorSpace")).start()
    }

    /// Write the `/Background` attribute.
    ///
    /// Sets the background color of the area to be shaded. The `background`
    /// iterator must contain exactly as many elements as the current
    /// color space has dimensions.
    pub fn background(&mut self, background: impl IntoIterator<Item = f32>) -> &mut Self {
        self.dict.insert(Name(b"Background")).array().items(background);
        self
    }

    /// Write the `/BBox` attribute.
    ///
    /// Sets the bounding box of the shading in the target coordinate system.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.dict.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/AntiAlias` attribute.
    ///
    /// Sets whether to anti-alias the shading.
    pub fn anti_alias(&mut self, anti_alias: bool) -> &mut Self {
        self.dict.pair(Name(b"AntiAlias"), anti_alias);
        self
    }

    /// Write the `/Domain` attribute.
    ///
    /// Sets the domain of the shading function in a rectangle. Can be used for
    /// function, axial, or radial shadings. Will otherwise default to
    /// `[x_min = 0, x_max = 1, y_min = 0, y_max = 1]`
    pub fn domain(&mut self, domain: [f32; 4]) -> &mut Self {
        self.dict.insert(Name(b"Domain")).array().items(domain);
        self
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Maps the shading domain rectangle to the target coordinate system. Can
    /// be used for function shadings. Will otherwise
    /// default to the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.dict.insert(Name(b"Matrix")).array().items(matrix);
        self
    }

    /// Write the `/Function` attribute.
    ///
    /// Sets the 2-in function to use for shading. Required.
    pub fn function(&mut self, function: Ref) -> &mut Self {
        self.dict.pair(Name(b"Function"), function);
        self
    }

    /// Write the `/Coords` attribute.
    ///
    /// Sets the coordinates of the start and end of the axis in terms of the
    /// target coordinate system. Required for axial (4 items) and radial (6
    /// items; centers and radii) shadings.
    pub fn coords(&mut self, coords: impl IntoIterator<Item = f32>) -> &mut Self {
        self.dict.insert(Name(b"Coords")).array().items(coords);
        self
    }

    /// Write the `/Extend` attribute.
    ///
    /// Set whether the shading should extend beyond either side of the axis /
    /// circles. Can be used for axial and radial shadings.
    pub fn extend(&mut self, extend: [bool; 2]) -> &mut Self {
        self.dict.insert(Name(b"Extend")).array().items(extend);
        self
    }
}

deref!('a, Shading<'a> => Dict<'a>, dict);

/// What kind of shading to use.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ShadingType {
    /// The function specifies the color for each point in the domain.
    Function,
    /// The function specifies the color for each point on a line.
    Axial,
    /// The function specifies the color for each circle between two nested circles.
    Radial,
}

impl ShadingType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Function => 1,
            Self::Axial => 2,
            Self::Radial => 3,
        }
    }
}

/// Writer for the _separation information dictionary_. PDF 1.3+.
///
/// This struct is created by [`Catalog::separation_info`].
pub struct SeparationInfo<'a> {
    dict: Dict<'a>,
}

writer!(SeparationInfo: |obj| Self { dict: obj.dict() });

impl SeparationInfo<'_> {
    /// Write the `/Pages` attribute. Required.
    ///
    /// This indicates all page dictionaries in the document that represent
    /// separations of the same page and shall be rendered together.
    pub fn pages(&mut self, pages: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.dict.insert(Name(b"Pages")).array().typed().items(pages);
        self
    }

    /// Write the `/DeviceColorant` attribute as a name. Required.
    ///
    /// The name of the device colorant that corresponds to the separation.
    pub fn device_colorant(&mut self, colorant: Name) -> &mut Self {
        self.dict.pair(Name(b"DeviceColorant"), colorant);
        self
    }

    /// Write the `/DeviceColorant` attribute as a string. Required.
    ///
    /// The name of the device colorant that corresponds to the separation.
    pub fn device_colorant_str(&mut self, colorant: &str) -> &mut Self {
        self.dict.pair(Name(b"DeviceColorant"), TextStr(colorant));
        self
    }

    /// Start writing the `/ColorSpace` array.
    ///
    /// This shall be an Separation or DeviceN color space that further defines
    /// the separation color space.
    pub fn color_space(&mut self) -> ColorSpace<'_> {
        self.dict.insert(Name(b"ColorSpace")).start()
    }
}

/// Writer for an _output intent dictionary_. PDF 1.4+.
///
/// This describes the output conditions under which the document may be
/// rendered.
pub struct OutputIntent<'a> {
    dict: Dict<'a>,
}

writer!(OutputIntent: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"OutputIntent"));
    Self { dict }
});

impl OutputIntent<'_> {
    /// Write the `/S` attribute. Required.
    pub fn subtype(&mut self, subtype: OutputIntentSubtype) -> &mut Self {
        self.dict.pair(Name(b"S"), subtype.to_name());
        self
    }

    /// Write the `/OutputCondition` attribute.
    ///
    /// A human-readable description of the output condition.
    pub fn output_condition(&mut self, condition: TextStr) -> &mut Self {
        self.dict.pair(Name(b"OutputCondition"), condition);
        self
    }

    /// Write the `/OutputConditionIdentifier` attribute.
    ///
    /// A well-known identifier for the output condition.
    pub fn output_condition_identifier(&mut self, identifier: TextStr) -> &mut Self {
        self.dict.pair(Name(b"OutputConditionIdentifier"), identifier);
        self
    }

    /// Write the `/RegistryName` attribute.
    ///
    /// The URI of the registry that contains the output condition.
    pub fn registry_name(&mut self, name: TextStr) -> &mut Self {
        self.dict.pair(Name(b"RegistryName"), name);
        self
    }

    /// Write the `/Info` attribute.
    ///
    /// A human-readable string with additional info about the intended output device.
    pub fn info(&mut self, info: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Info"), info);
        self
    }

    /// Write the `/DestOutputProfile` attribute.
    ///
    /// Required if `/OutputConditionIdentifier` does not contain a well-known
    /// identifier for the output condition.
    /// Must reference an [ICC profile](IccProfile) stream.
    pub fn dest_output_profile(&mut self, profile: Ref) -> &mut Self {
        self.dict.pair(Name(b"DestOutputProfile"), profile);
        self
    }
}

/// The output intent subtype.
pub enum OutputIntentSubtype<'a> {
    /// `GTS_PDFX`
    PDFX,
    /// `GTS_PDFA1`
    PDFA,
    /// `ISO_PDFE1`
    PDFE,
    /// Custom name defined in an ISO 32000 extension.
    Custom(Name<'a>),
}

impl<'a> OutputIntentSubtype<'a> {
    pub(crate) fn to_name(self) -> Name<'a> {
        match self {
            Self::PDFX => Name(b"GTS_PDFX"),
            Self::PDFA => Name(b"GTS_PDFA1"),
            Self::PDFE => Name(b"ISO_PDFE1"),
            Self::Custom(name) => name,
        }
    }
}
