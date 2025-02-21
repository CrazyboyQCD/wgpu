use crate::alloc::Vec;
use crate::diagnostic_filter::DiagnosticFilterNode;
use crate::front::wgsl::parse::directive::enable_extension::EnableExtensions;
use crate::front::wgsl::parse::number::Number;
use crate::front::wgsl::Scalar;
use crate::{Arena, FastIndexSet, Handle, Span};
use std::hash::Hash;
#[derive(Debug, Default)]
pub struct TranslationUnit<'alloc, 'a: 'alloc> {
    pub enable_extensions: EnableExtensions,
    pub decls: Arena<GlobalDecl<'alloc, 'a>>,
    /// The common expressions arena for the entire translation unit.
    ///
    /// All functions, global initializers, array lengths, etc. store their
    /// expressions here. We apportion these out to individual Naga
    /// [`Function`]s' expression arenas at lowering time. Keeping them all in a
    /// single arena simplifies handling of things like array lengths (which are
    /// effectively global and thus don't clearly belong to any function) and
    /// initializers (which can appear in both function-local and module-scope
    /// contexts).
    ///
    /// [`Function`]: crate::Function
    pub expressions: Arena<Expression<'alloc, 'a>>,

    /// Non-user-defined types, like `vec4<f32>` or `array<i32, 10>`.
    ///
    /// These are referred to by `Handle<ast::Type<'alloc,'a>>` values.
    /// User-defined types are referred to by name until lowering.
    pub types: Arena<Type<'alloc, 'a>>,

    /// Arena for all diagnostic filter rules parsed in this module, including those in functions.
    ///
    /// See [`DiagnosticFilterNode`] for details on how the tree is represented and used in
    /// validation.
    pub diagnostic_filters: Arena<DiagnosticFilterNode>,
    /// The leaf of all `diagnostic(…)` directives in this module.
    ///
    /// See [`DiagnosticFilterNode`] for details on how the tree is represented and used in
    /// validation.
    pub diagnostic_filter_leaf: Option<Handle<DiagnosticFilterNode>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Ident<'a> {
    pub name: &'a str,
    pub span: Span,
}

#[derive(Debug)]
pub enum IdentExpr<'a> {
    Unresolved(&'a str),
    Local(Handle<Local>),
}

/// A reference to a module-scope definition or predeclared object.
///
/// Each [`GlobalDecl`] holds a set of these values, to be resolved to
/// specific definitions later. To support de-duplication, `Eq` and
/// `Hash` on a `Dependency` value consider only the name, not the
/// source location at which the reference occurs.
#[derive(Debug)]
pub struct Dependency<'a> {
    /// The name referred to.
    pub ident: &'a str,

    /// The location at which the reference to that name occurs.
    pub usage: Span,
}

impl Hash for Dependency<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}

impl PartialEq for Dependency<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for Dependency<'_> {}

/// A module-scope declaration.
#[derive(Debug)]
pub struct GlobalDecl<'alloc, 'a: 'alloc> {
    pub kind: GlobalDeclKind<'alloc, 'a>,

    /// Names of all module-scope or predeclared objects this
    /// declaration uses.
    pub dependencies: FastIndexSet<Dependency<'a>>,
}

#[derive(Debug)]
pub enum GlobalDeclKind<'alloc, 'a: 'alloc> {
    Fn(Function<'alloc, 'a>),
    Var(GlobalVariable<'alloc, 'a>),
    Const(Const<'alloc, 'a>),
    Override(Override<'alloc, 'a>),
    Struct(Struct<'alloc, 'a>),
    Type(TypeAlias<'alloc, 'a>),
    ConstAssert(Handle<Expression<'alloc, 'a>>),
}

#[derive(Debug)]
pub struct FunctionArgument<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Handle<Type<'alloc, 'a>>,
    pub binding: Option<Binding<'alloc, 'a>>,
    pub handle: Handle<Local>,
}

#[derive(Debug)]
pub struct FunctionResult<'alloc, 'a: 'alloc> {
    pub ty: Handle<Type<'alloc, 'a>>,
    pub binding: Option<Binding<'alloc, 'a>>,
    pub must_use: bool,
}

#[derive(Debug)]
pub struct EntryPoint<'alloc, 'a: 'alloc> {
    pub stage: crate::ShaderStage,
    pub early_depth_test: Option<crate::EarlyDepthTest>,
    pub workgroup_size: Option<[Option<Handle<Expression<'alloc, 'a>>>; 3]>,
}

#[cfg(doc)]
use crate::front::wgsl::lower::{LocalExpressionContext, StatementContext};

#[derive(Debug)]
pub struct Function<'alloc, 'a: 'alloc> {
    pub entry_point: Option<EntryPoint<'alloc, 'a>>,
    pub name: Ident<'a>,
    pub arguments: Vec<'alloc, FunctionArgument<'alloc, 'a>>,
    pub result: Option<FunctionResult<'alloc, 'a>>,
    pub body: Block<'alloc, 'a>,
    pub diagnostic_filter_leaf: Option<Handle<DiagnosticFilterNode>>,
}

#[derive(Debug)]
pub enum Binding<'alloc, 'a: 'alloc> {
    BuiltIn(crate::BuiltIn),
    Location {
        location: Handle<Expression<'alloc, 'a>>,
        second_blend_source: bool,
        interpolation: Option<crate::Interpolation>,
        sampling: Option<crate::Sampling>,
    },
}

#[derive(Debug)]
pub struct ResourceBinding<'alloc, 'a: 'alloc> {
    pub group: Handle<Expression<'alloc, 'a>>,
    pub binding: Handle<Expression<'alloc, 'a>>,
}

#[derive(Debug)]
pub struct GlobalVariable<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub space: crate::AddressSpace,
    pub binding: Option<ResourceBinding<'alloc, 'a>>,
    pub ty: Option<Handle<Type<'alloc, 'a>>>,
    pub init: Option<Handle<Expression<'alloc, 'a>>>,
}

#[derive(Debug)]
pub struct StructMember<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Handle<Type<'alloc, 'a>>,
    pub binding: Option<Binding<'alloc, 'a>>,
    pub align: Option<Handle<Expression<'alloc, 'a>>>,
    pub size: Option<Handle<Expression<'alloc, 'a>>>,
}

#[derive(Debug)]
pub struct Struct<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub members: Vec<'alloc, StructMember<'alloc, 'a>>,
}

#[derive(Debug)]
pub struct TypeAlias<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Handle<Type<'alloc, 'a>>,
}

#[derive(Debug)]
pub struct Const<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Option<Handle<Type<'alloc, 'a>>>,
    pub init: Handle<Expression<'alloc, 'a>>,
}

#[derive(Debug)]
pub struct Override<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub id: Option<Handle<Expression<'alloc, 'a>>>,
    pub ty: Option<Handle<Type<'alloc, 'a>>>,
    pub init: Option<Handle<Expression<'alloc, 'a>>>,
}

/// The size of an [`Array`] or [`BindingArray`].
///
/// [`Array`]: Type::Array
/// [`BindingArray`]: Type::BindingArray
#[derive(Debug, Copy, Clone)]
pub enum ArraySize<'alloc, 'a: 'alloc> {
    /// The length as a constant expression.
    Constant(Handle<Expression<'alloc, 'a>>),
    Dynamic,
}

#[derive(Debug)]
pub enum Type<'alloc, 'a: 'alloc> {
    Scalar(Scalar),
    Vector {
        size: crate::VectorSize,
        ty: Handle<Type<'alloc, 'a>>,
        ty_span: Span,
    },
    Matrix {
        columns: crate::VectorSize,
        rows: crate::VectorSize,
        ty: Handle<Type<'alloc, 'a>>,
        ty_span: Span,
    },
    Atomic(Scalar),
    Pointer {
        base: Handle<Type<'alloc, 'a>>,
        space: crate::AddressSpace,
    },
    Array {
        base: Handle<Type<'alloc, 'a>>,
        size: ArraySize<'alloc, 'a>,
    },
    Image {
        dim: crate::ImageDimension,
        arrayed: bool,
        class: crate::ImageClass,
    },
    Sampler {
        comparison: bool,
    },
    AccelerationStructure,
    RayQuery,
    RayDesc,
    RayIntersection,
    BindingArray {
        base: Handle<Type<'alloc, 'a>>,
        size: ArraySize<'alloc, 'a>,
    },

    /// A user-defined type, like a struct or a type alias.
    User(Ident<'a>),
}

#[derive(Debug, Default)]
pub struct Block<'alloc, 'a: 'alloc> {
    pub stmts: Vec<'alloc, Statement<'alloc, 'a>>,
}

#[derive(Debug)]
pub struct Statement<'alloc, 'a: 'alloc> {
    pub kind: StatementKind<'alloc, 'a>,
    pub span: Span,
}

#[derive(Debug)]
pub enum StatementKind<'alloc, 'a: 'alloc> {
    LocalDecl(LocalDecl<'alloc, 'a>),
    Block(Block<'alloc, 'a>),
    If {
        condition: Handle<Expression<'alloc, 'a>>,
        accept: Block<'alloc, 'a>,
        reject: Block<'alloc, 'a>,
    },
    Switch {
        selector: Handle<Expression<'alloc, 'a>>,
        cases: Vec<'alloc, SwitchCase<'alloc, 'a>>,
    },
    Loop {
        body: Block<'alloc, 'a>,
        continuing: Block<'alloc, 'a>,
        break_if: Option<Handle<Expression<'alloc, 'a>>>,
    },
    Break,
    Continue,
    Return {
        value: Option<Handle<Expression<'alloc, 'a>>>,
    },
    Kill,
    Call {
        function: Ident<'a>,
        arguments: Vec<'alloc, Handle<Expression<'alloc, 'a>>>,
    },
    Assign {
        target: Handle<Expression<'alloc, 'a>>,
        op: Option<crate::BinaryOperator>,
        value: Handle<Expression<'alloc, 'a>>,
    },
    Increment(Handle<Expression<'alloc, 'a>>),
    Decrement(Handle<Expression<'alloc, 'a>>),
    Phony(Handle<Expression<'alloc, 'a>>),
    ConstAssert(Handle<Expression<'alloc, 'a>>),
}

#[derive(Debug)]
pub enum SwitchValue<'alloc, 'a: 'alloc> {
    Expr(Handle<Expression<'alloc, 'a>>),
    Default,
}

#[derive(Debug)]
pub struct SwitchCase<'alloc, 'a: 'alloc> {
    pub value: SwitchValue<'alloc, 'a>,
    pub body: Block<'alloc, 'a>,
    pub fall_through: bool,
}

/// A type at the head of a [`Construct`] expression.
///
/// WGSL has two types of [`type constructor expressions`]:
///
/// - Those that fully specify the type being constructed, like
///   `vec3<f32>(x,y,z)`, which obviously constructs a `vec3<f32>`.
///
/// - Those that leave the component type of the composite being constructed
///   implicit, to be inferred from the argument types, like `vec3(x,y,z)`,
///   which constructs a `vec3<T>` where `T` is the type of `x`, `y`, and `z`.
///
/// This enum represents the head type of both cases. The `PartialFoo` variants
/// represent the second case, where the component type is implicit.
///
/// This does not cover structs or types referred to by type aliases. See the
/// documentation for [`Construct`] and [`Call`] expressions for details.
///
/// [`Construct`]: Expression::Construct
/// [`type constructor expressions`]: https://gpuweb.github.io/gpuweb/wgsl/#type-constructor-expr
/// [`Call`]: Expression::Call
#[derive(Debug)]
pub enum ConstructorType<'alloc, 'a: 'alloc> {
    /// A scalar type or conversion: `f32(1)`.
    Scalar(Scalar),

    /// A vector construction whose component type is inferred from the
    /// argument: `vec3(1.0)`.
    PartialVector { size: crate::VectorSize },

    /// A vector construction whose component type is written out:
    /// `vec3<f32>(1.0)`.
    Vector {
        size: crate::VectorSize,
        ty: Handle<Type<'alloc, 'a>>,
        ty_span: Span,
    },

    /// A matrix construction whose component type is inferred from the
    /// argument: `mat2x2(1,2,3,4)`.
    PartialMatrix {
        columns: crate::VectorSize,
        rows: crate::VectorSize,
    },

    /// A matrix construction whose component type is written out:
    /// `mat2x2<f32>(1,2,3,4)`.
    Matrix {
        columns: crate::VectorSize,
        rows: crate::VectorSize,
        ty: Handle<Type<'alloc, 'a>>,
        ty_span: Span,
    },

    /// An array whose component type and size are inferred from the arguments:
    /// `array(3,4,5)`.
    PartialArray,

    /// An array whose component type and size are written out:
    /// `array<u32, 4>(3,4,5)`.
    Array {
        base: Handle<Type<'alloc, 'a>>,
        size: ArraySize<'alloc, 'a>,
    },

    /// Constructing a value of a known Naga IR type.
    ///
    /// This variant is produced only during lowering, when we have Naga types
    /// available, never during parsing.
    Type(Handle<crate::Type>),
}

#[derive(Debug, Copy, Clone)]
pub enum Literal {
    Bool(bool),
    Number(Number),
}

#[cfg(doc)]
use crate::front::wgsl::lower::Lowerer;

#[derive(Debug)]
pub enum Expression<'alloc, 'a: 'alloc> {
    Literal(Literal),
    Ident(IdentExpr<'a>),

    /// A type constructor expression.
    ///
    /// This is only used for expressions like `KEYWORD(EXPR...)` and
    /// `KEYWORD<PARAM>(EXPR...)`, where `KEYWORD` is a [type-defining keyword] like
    /// `vec3`. These keywords cannot be shadowed by user definitions, so we can
    /// tell that such an expression is a construction immediately.
    ///
    /// For ordinary identifiers, we can't tell whether an expression like
    /// `IDENTIFIER(EXPR, ...)` is a construction expression or a function call
    /// until we know `IDENTIFIER`'s definition, so we represent those as
    /// [`Call`] expressions.
    ///
    /// [type-defining keyword]: https://gpuweb.github.io/gpuweb/wgsl/#type-defining-keywords
    /// [`Call`]: Expression::Call
    Construct {
        ty: ConstructorType<'alloc, 'a>,
        ty_span: Span,
        components: Vec<'alloc, Handle<Expression<'alloc, 'a>>>,
    },
    Unary {
        op: crate::UnaryOperator,
        expr: Handle<Expression<'alloc, 'a>>,
    },
    AddrOf(Handle<Expression<'alloc, 'a>>),
    Deref(Handle<Expression<'alloc, 'a>>),
    Binary {
        op: crate::BinaryOperator,
        left: Handle<Expression<'alloc, 'a>>,
        right: Handle<Expression<'alloc, 'a>>,
    },

    /// A function call or type constructor expression.
    ///
    /// We can't tell whether an expression like `IDENTIFIER(EXPR, ...)` is a
    /// construction expression or a function call until we know `IDENTIFIER`'s
    /// definition, so we represent everything of that form as one of these
    /// expressions until lowering. At that point, [`Lowerer::call`] has
    /// everything's definition in hand, and can decide whether to emit a Naga
    /// [`Constant`], [`As`], [`Splat`], or [`Compose`] expression.
    ///
    /// [`Lowerer::call`]: Lowerer::call
    /// [`Constant`]: crate::Expression::Constant
    /// [`As`]: crate::Expression::As
    /// [`Splat`]: crate::Expression::Splat
    /// [`Compose`]: crate::Expression::Compose
    Call {
        function: Ident<'a>,
        arguments: Vec<'alloc, Handle<Expression<'alloc, 'a>>>,
    },
    Index {
        base: Handle<Expression<'alloc, 'a>>,
        index: Handle<Expression<'alloc, 'a>>,
    },
    Member {
        base: Handle<Expression<'alloc, 'a>>,
        field: Ident<'a>,
    },
    Bitcast {
        expr: Handle<Expression<'alloc, 'a>>,
        to: Handle<Type<'alloc, 'a>>,
        ty_span: Span,
    },
}

#[derive(Debug)]
pub struct LocalVariable<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Option<Handle<Type<'alloc, 'a>>>,
    pub init: Option<Handle<Expression<'alloc, 'a>>>,
    pub handle: Handle<Local>,
}

#[derive(Debug)]
pub struct Let<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Option<Handle<Type<'alloc, 'a>>>,
    pub init: Handle<Expression<'alloc, 'a>>,
    pub handle: Handle<Local>,
}

#[derive(Debug)]
pub struct LocalConst<'alloc, 'a: 'alloc> {
    pub name: Ident<'a>,
    pub ty: Option<Handle<Type<'alloc, 'a>>>,
    pub init: Handle<Expression<'alloc, 'a>>,
    pub handle: Handle<Local>,
}

#[derive(Debug)]
pub enum LocalDecl<'alloc, 'a: 'alloc> {
    Var(LocalVariable<'alloc, 'a>),
    Let(Let<'alloc, 'a>),
    Const(LocalConst<'alloc, 'a>),
}

#[derive(Debug)]
/// A placeholder for a local variable declaration.
///
/// See [`super::ExpressionContext::locals`] for more information.
pub struct Local;
