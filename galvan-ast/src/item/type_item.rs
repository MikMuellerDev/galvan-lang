use derive_more::From;

use galvan_pest::Rule;

use crate::TypeIdent;

#[derive(Debug, PartialEq, Eq, From, FromPest)]
#[pest_ast(rule(Rule::type_item))]
pub enum TypeElement {
    // Collection Types
    Array(Box<ArrayTypeItem>),
    Dictionary(Box<DictionaryTypeItem>),
    OrderedDictionary(Box<OrderedDictionaryTypeItem>),
    Set(Box<SetTypeItem>),
    Tuple(Box<TupleTypeItem>),

    // Error handling monads
    Optional(Box<OptionalTypeItem>),
    Result(Box<ResultTypeItem>),

    // Primitive type
    Plain(BasicTypeItem),
}

impl From<TypeIdent> for TypeElement {
    fn from(value: TypeIdent) -> Self {
        Self::Plain(BasicTypeItem { ident: value })
    }
}

impl TypeElement {
    pub fn plain(ident: TypeIdent) -> Self {
        Self::Plain(BasicTypeItem { ident })
    }

    pub fn array(elements: TypeElement) -> Self {
        Self::Array(Box::new(ArrayTypeItem { elements }))
    }

    pub fn dict(key: TypeElement, value: TypeElement) -> Self {
        Self::Dictionary(Box::new(DictionaryTypeItem { key, value }))
    }

    pub fn ordered_dict(key: TypeElement, value: TypeElement) -> Self {
        Self::OrderedDictionary(Box::new(OrderedDictionaryTypeItem { key, value }))
    }

    pub fn set(elements: TypeElement) -> Self {
        Self::Set(Box::new(SetTypeItem { elements }))
    }

    pub fn tuple(elements: Vec<TypeElement>) -> Self {
        Self::Tuple(Box::new(TupleTypeItem { elements }))
    }

    pub fn optional(some: OptionalElement) -> Self {
        Self::Optional(Box::new(OptionalTypeItem { some }))
    }

    pub fn result(success: SuccessVariant, error: Option<ErrorVariant>) -> Self {
        Self::Result(Box::new(ResultTypeItem { success, error }))
    }
}

// TODO: Add a marker trait to constrain this to only type decls
#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::array_type))]
pub struct ArrayTypeItem {
    pub elements: TypeElement,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::dict_type))]
pub struct DictionaryTypeItem {
    pub key: TypeElement,
    pub value: TypeElement,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::ordered_dict_type))]
pub struct OrderedDictionaryTypeItem {
    pub key: TypeElement,
    pub value: TypeElement,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::set_type))]
pub struct SetTypeItem {
    pub elements: TypeElement,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::tuple_type))]
pub struct TupleTypeItem {
    pub elements: Vec<TypeElement>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::optional_type))]
pub struct OptionalTypeItem {
    some: OptionalElement,
}

impl OptionalTypeItem {
    pub fn new(some: OptionalElement) -> Self {
        Self { some }
    }

    /// Lifts the inner type of the optional to a type element
    pub fn element(self) -> TypeElement {
        self.some.into()
    }
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::opt_element_type))]
/// A subset of TypeElement that can be used as the inner type of an optional
pub enum OptionalElement {
    Array(Box<ArrayTypeItem>),
    Dictionary(Box<DictionaryTypeItem>),
    OrderedDictionary(Box<OrderedDictionaryTypeItem>),
    Set(Box<SetTypeItem>),
    Tuple(Box<TupleTypeItem>),
    Plain(BasicTypeItem),
}

impl From<OptionalElement> for TypeElement {
    fn from(value: OptionalElement) -> Self {
        match value {
            OptionalElement::Array(array) => Self::Array(array),
            OptionalElement::Dictionary(dict) => Self::Dictionary(dict),
            OptionalElement::OrderedDictionary(ordered_dict) => {
                Self::OrderedDictionary(ordered_dict)
            }
            OptionalElement::Set(set) => Self::Set(set),
            OptionalElement::Tuple(tuple) => Self::Tuple(tuple),
            OptionalElement::Plain(basic) => Self::Plain(basic),
        }
    }
}

impl TryFrom<TypeElement> for OptionalElement {
    type Error = TypeElement;

    fn try_from(value: TypeElement) -> Result<Self, Self::Error> {
        match value {
            TypeElement::Array(array) => Ok(Self::Array(array)),
            TypeElement::Dictionary(dict) => Ok(Self::Dictionary(dict)),
            TypeElement::OrderedDictionary(ordered_dict) => {
                Ok(Self::OrderedDictionary(ordered_dict))
            }
            TypeElement::Set(set) => Ok(Self::Set(set)),
            TypeElement::Tuple(tuple) => Ok(Self::Tuple(tuple)),
            TypeElement::Plain(basic) => Ok(Self::Plain(basic)),
            TypeElement::Optional(_) => Err(value),
            TypeElement::Result(_) => Err(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::result_type))]
pub struct ResultTypeItem {
    success: SuccessVariant,
    error: Option<ErrorVariant>,
}

pub struct DowncastResultTypeItem {
    pub success: TypeElement,
    pub error: Option<TypeElement>,
}

impl ResultTypeItem {
    pub fn new(success: SuccessVariant, error: Option<ErrorVariant>) -> Self {
        Self { success, error }
    }
}

impl From<ResultTypeItem> for DowncastResultTypeItem {
    /// Lifts the success and error variant to a TypeElement
    fn from(value: ResultTypeItem) -> Self {
        let ResultTypeItem { success, error } = value;
        let success = TypeElement::from(success);
        let error = error.map(TypeElement::from);
        Self { success, error }
    }
}

#[derive(Debug, PartialEq, Eq, From, FromPest)]
#[pest_ast(rule(Rule::success_variant))]
pub enum SuccessVariant {
    Array(Box<ArrayTypeItem>),
    Dictionary(Box<DictionaryTypeItem>),
    OrderedDictionary(Box<OrderedDictionaryTypeItem>),
    Set(Box<SetTypeItem>),
    Tuple(Box<TupleTypeItem>),
    Plain(BasicTypeItem),
    Optional(Box<OptionalTypeItem>),
}

impl From<SuccessVariant> for TypeElement {
    fn from(value: SuccessVariant) -> Self {
        match value {
            SuccessVariant::Array(array) => Self::Array(array),
            SuccessVariant::Dictionary(dict) => Self::Dictionary(dict),
            SuccessVariant::OrderedDictionary(ordered_dict) => {
                Self::OrderedDictionary(ordered_dict)
            }
            SuccessVariant::Set(set) => Self::Set(set),
            SuccessVariant::Tuple(tuple) => Self::Tuple(tuple),
            SuccessVariant::Plain(basic) => Self::Plain(basic),
            SuccessVariant::Optional(optional) => Self::Optional(optional),
        }
    }
}

#[derive(Debug, PartialEq, Eq, From, FromPest)]
#[pest_ast(rule(Rule::error_variant))]
pub enum ErrorVariant {
    Array(Box<ArrayTypeItem>),
    Dictionary(Box<DictionaryTypeItem>),
    OrderedDictionary(Box<OrderedDictionaryTypeItem>),
    Set(Box<SetTypeItem>),
    Tuple(Box<TupleTypeItem>),
    Plain(BasicTypeItem),
}

impl From<ErrorVariant> for TypeElement {
    fn from(value: ErrorVariant) -> Self {
        match value {
            ErrorVariant::Array(array) => Self::Array(array),
            ErrorVariant::Dictionary(dict) => Self::Dictionary(dict),
            ErrorVariant::OrderedDictionary(ordered_dict) => Self::OrderedDictionary(ordered_dict),
            ErrorVariant::Set(set) => Self::Set(set),
            ErrorVariant::Tuple(tuple) => Self::Tuple(tuple),
            ErrorVariant::Plain(basic) => Self::Plain(basic),
        }
    }
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::basic_type))]
pub struct BasicTypeItem {
    pub ident: TypeIdent,
    // TODO: Handle generics
}

#[cfg(test)]
mod test {
    use crate::item::string;
    use from_pest::pest::Parser;
    use from_pest::FromPest;
    use galvan_pest::Rule;

    use super::*;

    fn partial_ast<'p, T>(src: &'p str, rule: Rule) -> Result<T, String>
    where
        T: FromPest<'p, Rule = Rule>,
    {
        let pairs = galvan_pest::GalvanParser::parse(rule, src).unwrap();
        T::from_pest(&mut pairs.clone())
            .map_err(|_| format!("Error when converting into ast!\n\n{pairs:#?}"))
    }

    #[test]
    fn test_plain_type() {
        let parsed: TypeElement =
            partial_ast("Int", Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
        let TypeElement::Plain(basic) = parsed else {
            panic!("Expected plain type")
        };
        assert_eq!(basic.ident, TypeIdent::new("Int"));
    }

    macro_rules! test_collection_type {
        ($lit:literal, $name:ident, $rule:ident, $variant:ident, $inner:ident) => {
            #[test]
            fn $name() {
                let parsed: $inner =
                    partial_ast($lit, Rule::$rule).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::Plain(elements) = parsed.elements else {
                    panic!("Expected plain type as element type")
                };
                assert_eq!(
                    elements.ident,
                    TypeIdent::new("Int"),
                    "Tested {} type",
                    stringify!($inner)
                );

                let parsed: TypeElement =
                    partial_ast($lit, Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::$variant(container) = parsed else {
                    panic!("Wrong type")
                };
                let TypeElement::Plain(elements) = container.elements else {
                    panic!("Expected plain type")
                };
                assert_eq!(elements.ident, TypeIdent::new("Int"), "Tested TypeItem");
            }
        };
    }

    test_collection_type!("[Int]", test_array_type, array_type, Array, ArrayTypeItem);
    test_collection_type!("{Int}", test_set_type, set_type, Set, SetTypeItem);

    #[test]
    fn test_tuple_type() {
        let parsed: TupleTypeItem =
            partial_ast("(Int, Float)", Rule::tuple_type).unwrap_or_else(|e| panic!("{}", e));
        let TypeElement::Plain(ref elements) = parsed.elements[0] else {
            panic!("Expected plain type as first element!")
        };
        assert_eq!(
            elements.ident,
            TypeIdent::new("Int"),
            "Testing first element"
        );
        let TypeElement::Plain(ref elements) = parsed.elements[1] else {
            panic!("Expected plain type as second element!")
        };
        assert_eq!(
            elements.ident,
            TypeIdent::new("Float"),
            "Testing second element"
        );

        let parsed: TypeElement =
            partial_ast("(Int, String)", Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
        let TypeElement::Tuple(container) = parsed else {
            panic!("Wrong type")
        };
        let TypeElement::Plain(ref elements) = container.elements[0] else {
            panic!("Expected plain type as first element!")
        };
        assert_eq!(
            elements.ident,
            TypeIdent::new("Int"),
            "Testing first element for downcast TypeItem"
        );
        let TypeElement::Plain(ref elements) = container.elements[1] else {
            panic!("Expected plain type as second element!")
        };
        assert_eq!(
            elements.ident,
            TypeIdent::new("String"),
            "Testing second element for downcast TypeItem"
        );
    }

    macro_rules! test_dictionary_type {
        ($lit:literal, $name:ident, $rule:ident, $variant:ident, $inner:ident) => {
            #[test]
            fn $name() {
                let parsed: $inner =
                    partial_ast($lit, Rule::$rule).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::Plain(key) = parsed.key else {
                    panic!("Expected plain type as key type")
                };
                assert_eq!(key.ident, TypeIdent::new("Int"), "Testing key");
                let TypeElement::Plain(value) = parsed.value else {
                    panic!("Expected plain type as value type")
                };
                assert_eq!(value.ident, TypeIdent::new("Float"), "Testing value");

                let parsed: TypeElement =
                    partial_ast($lit, Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::$variant(container) = parsed else {
                    panic!("Wrong type")
                };
                let TypeElement::Plain(key) = container.key else {
                    panic!("Expected plain type as key type")
                };
                assert_eq!(
                    key.ident,
                    TypeIdent::new("Int"),
                    "Testing key for downcast TypeItem"
                );
                let TypeElement::Plain(value) = container.value else {
                    panic!("Expected plain type as value type")
                };
                assert_eq!(
                    value.ident,
                    TypeIdent::new("Float"),
                    "Testing value for downcast TypeItem"
                );
            }
        };
    }

    test_dictionary_type!(
        "{Int: Float}",
        test_dict_type,
        dict_type,
        Dictionary,
        DictionaryTypeItem
    );
    test_dictionary_type!(
        "[Int: Float]",
        test_ordered_dict_type,
        ordered_dict_type,
        OrderedDictionary,
        OrderedDictionaryTypeItem
    );

    #[test]
    fn test_optional_type() {
        let parsed: OptionalTypeItem =
            partial_ast("Int?", Rule::optional_type).unwrap_or_else(|e| panic!("{}", e));
        let TypeElement::Plain(some) = parsed.element() else {
            panic!("Expected plain type as some type")
        };
        assert_eq!(some.ident, TypeIdent::new("Int"), "Testing some");

        let parsed: TypeElement =
            partial_ast("Int?", Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
        let TypeElement::Optional(container) = parsed else {
            panic!("Wrong type")
        };
        let TypeElement::Plain(some) = container.element() else {
            panic!("Expected plain type as some type")
        };
        assert_eq!(
            some.ident,
            TypeIdent::new("Int"),
            "Testing some for downcast TypeItem"
        );
    }

    macro_rules! test_optional_dictionary_type {
        ($lit:literal, $name:ident, $rule:ident, $variant:ident, $inner:ident) => {
            #[test]
            fn $name() {
                let parsed: OptionalTypeItem =
                    partial_ast($lit, Rule::optional_type).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::$variant(container) = parsed.element() else {
                    panic!("Expected {} type as some type", stringify!($variant))
                };

                let TypeElement::Plain(key) = container.key else {
                    panic!("Expected plain type as key type")
                };
                assert_eq!(key.ident, TypeIdent::new("Int"), "Testing key");
                let TypeElement::Plain(value) = container.value else {
                    panic!("Expected plain type as value type")
                };
                assert_eq!(value.ident, TypeIdent::new("Float"), "Testing value");

                let parsed: TypeElement =
                    partial_ast($lit, Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::Optional(container) = parsed else {
                    panic!("Wrong type")
                };
                let TypeElement::$variant(container) = container.element() else {
                    panic!("Expected {} type as some type", stringify!($variant))
                };
                let TypeElement::Plain(key) = container.key else {
                    panic!("Expected plain type as key type")
                };
                assert_eq!(
                    key.ident,
                    TypeIdent::new("Int"),
                    "Testing key for downcast TypeItem"
                );
                let TypeElement::Plain(value) = container.value else {
                    panic!("Expected plain type as value type")
                };
                assert_eq!(
                    value.ident,
                    TypeIdent::new("Float"),
                    "Testing value for downcast TypeItem"
                );
            }
        };
    }

    test_optional_dictionary_type!(
        "{Int: Float}?",
        test_optional_dict_type,
        optional_dict_type,
        Dictionary,
        DictionaryTypeItem
    );
    test_optional_dictionary_type!(
        "[Int: Float]?",
        test_optional_ordered_dict_type,
        optional_ordered_dict_type,
        OrderedDictionary,
        OrderedDictionaryTypeItem
    );

    macro_rules! optional_collection_type {
        ($lit:literal, $name:ident, $rule:ident, $variant:ident, $inner:ident) => {
            #[test]
            fn $name() {
                let parsed: OptionalTypeItem =
                    partial_ast($lit, Rule::optional_type).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::$variant(container) = parsed.element() else {
                    panic!("Expected {} type as some type", stringify!($variant))
                };
                let TypeElement::Plain(elements) = container.elements else {
                    panic!("Expected plain type as element type")
                };
                assert_eq!(elements.ident, TypeIdent::new("Int"), "Testing elements");

                let parsed: TypeElement =
                    partial_ast($lit, Rule::type_item).unwrap_or_else(|e| panic!("{}", e));
                let TypeElement::Optional(container) = parsed else {
                    panic!("Wrong type")
                };
                let TypeElement::$variant(container) = container.element() else {
                    panic!("Expected {} type as some type", stringify!($variant))
                };
                let TypeElement::Plain(elements) = container.elements else {
                    panic!("Expected plain type as element type")
                };
                assert_eq!(
                    elements.ident,
                    TypeIdent::new("Int"),
                    "Testing elements for downcast TypeItem"
                );
            }
        };
    }

    optional_collection_type!(
        "[Int]?",
        test_optional_array_type,
        optional_array_type,
        Array,
        ArrayTypeItem
    );
    optional_collection_type!(
        "{Int}?",
        test_optional_set_type,
        optional_set_type,
        Set,
        SetTypeItem
    );
}