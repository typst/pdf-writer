use super::*;

/// The type of a color space.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(unused)]
enum ColorSpaceType {
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
}

/// Writer for a _color space_.
///
/// This struct is created by [`PdfWriter::color_space`],
/// [`Shading::color_space`], [`ImageXObject::color_space`] and
/// [`Group::color_space`].
pub struct ColorSpace<'a> {
    obj: Obj<'a>,
}

impl<'a> Writer<'a> for ColorSpace<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { obj }
    }
}

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

    /// Write a `CalRGB` color space for sRGB.
    pub fn srgb(self) {
        self.cal_rgb(
            [0.9505, 1.0, 1.089],
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.4124, 0.2126, 0.0193, 0.3576, 0.715, 0.1192, 0.1805, 0.0722, 0.9505,
            ]),
        )
    }

    /// Write a `CalRGB` color space for Adobe RGB.
    pub fn adobe_rgb(self) {
        self.cal_rgb(
            [0.9505, 1.0, 1.089],
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.76670, 0.29734, 0.02703, 0.18556, 0.62736, 0.07069, 0.18823, 0.07529,
                0.99134,
            ]),
        )
    }

    /// Write a `CalRGB` color space for Display P3.
    pub fn display_p3(self) {
        self.cal_rgb(
            [0.9505, 1.0, 1.089],
            None,
            Some([2.2, 2.2, 2.2]),
            Some([
                0.48657, 0.2297, 0.0, 0.26567, 0.69174, 0.04511, 0.19822, 0.07929,
                1.04394,
            ]),
        )
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

    /// Write a `CalGray` color space for CIE D65 at a 2.2 gamma, equivalent to
    /// sRGB.
    pub fn srgb_gray(self) {
        self.cal_gray([0.9505, 1.0, 1.089], None, Some(2.2))
    }

    /// Write a `CalGray` color space for Adobe RGB.
    pub fn adobe_rgb_gray(self) {
        self.cal_gray([0.9505, 1.0, 1.089], None, Some(2.2))
    }

    /// Write a `CalGray` color space for Display P3.
    pub fn display_p3_gray(self) {
        self.cal_gray([0.9505, 1.0, 1.089], None, Some(2.2))
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

    // TODO: ICC-based.
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
impl ColorSpace<'_> {
    /// Write a `Separation` color space. PDF 1.2+.
    pub fn separation(self, color_name: Name, base: Name, tint: Ref) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::Separation.to_name());
        array.item(color_name);
        array.item(base);
        array.item(tint);
    }

    /// Write a `DeviceN` color space. PDF 1.3+.
    pub fn device_n<'n>(
        self,
        names: impl IntoIterator<Item = Name<'n>>,
        alternate_space: Name,
        tint: Ref,
    ) {
        let mut array = self.obj.array();
        array.item(ColorSpaceType::DeviceN.to_name());
        array.push().array().items(names);
        array.item(alternate_space);
        array.item(tint);
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
    /// The `base` attribute is the color space in which the pattern color is
    /// specified upon use.
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

/// Writer for a _tiling pattern stream_.
///
/// This struct is created by [`PdfWriter::tiling_pattern`].
pub struct TilingPattern<'a> {
    stream: Stream<'a>,
}

impl<'a> TilingPattern<'a> {
    /// Create a new tiling pattern writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
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

impl<'a> Writer<'a> for ShadingPattern<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Pattern"));
        dict.pair(Name(b"PatternType"), PatternType::Shading.to_int());
        Self { dict }
    }
}

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

    /// Begin writing the `/ExtGState` attribute.
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

impl<'a> Writer<'a> for Shading<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }
}

impl<'a> Shading<'a> {
    /// Write the `/ShadingType` attribute.
    ///
    /// Sets the type of shading. The available and required attributes change
    /// depending on this. Required.
    pub fn shading_type(&mut self, shading_type: ShadingType) -> &mut Self {
        self.dict.pair(Name(b"ShadingType"), shading_type.to_int());
        self
    }

    /// Write the `/ColorSpace` attribute.
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
