use std::borrow::Cow;

use cosmic::{
    iced::{alignment, Color, Length},
    iced_widget::{pick_list, toggler},
    prelude::CollectionWidget,
    widget::{
        button, column, container, horizontal_space, mouse_area, row,
        segmented_button::Entity,
        settings::section,
        text, text_input,
        tooltip::{tooltip, Position},
    },
    Element,
};

use crate::{
    app::App,
    icon,
    message::{AppMsg, ChangeMsg, PageMsg},
    node::{
        data_path::{DataPath, DataPathType},
        Node, NodeArray, NodeBool, NodeContainer, NodeEnum, NodeNumber, NodeObject, NodeString,
        NodeValue, NumberKind,
    },
    page::Page,
};

const SPACING: f32 = 10.;

pub fn view_app(app: &App) -> Element<'_, AppMsg> {
    let entity = app.nav_model.active();
    let page = app.nav_model.data::<Page>(entity).unwrap();

    container(view_page(entity, page).map(move |msg| AppMsg::PageMsg(entity, msg)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_data_path(data_path: &DataPath) -> Element<'_, PageMsg> {
    let mut elements = Vec::new();

    let get_class = |pos: Option<usize>| {
        if pos == data_path.pos {
            button::ButtonClass::Text
        } else {
            button::ButtonClass::MenuRoot
        }
    };

    elements.push(
        button::text("/".to_string())
            .on_press(PageMsg::SelectDataPath(None))
            .class(get_class(None))
            .into(),
    );

    for (pos, component) in data_path.vec.iter().enumerate() {
        elements.push(
            button::text(format!("{}", component))
                .on_press(PageMsg::SelectDataPath(Some(pos)))
                .class(get_class(Some(pos)))
                .into(),
        );
    }

    row::with_children(elements).into()
}

fn view_page(entity: Entity, page: &Page) -> Element<'_, PageMsg> {
    let data_path = page.data_path.current();

    let node = page.tree.get_at(data_path.iter()).unwrap();

    let content = match &node.node {
        Node::Bool(node_bool) => view_bool(data_path, node, node_bool),
        Node::String(node_string) => view_string(data_path, node, node_string),
        Node::Number(node_number) => view_number(data_path, node, node_number),
        Node::Object(node_object) => view_object(data_path, node, node_object),
        Node::Enum(node_enum) => view_enum(data_path, node, node_enum),
        Node::Value(node_value) => view_value(data_path, node, node_value),
        Node::Null => text("null").into(),
        Node::Array(node_array) => view_array(data_path, node, node_array),
    };

    column()
        .push(view_data_path(&page.data_path))
        .push(content)
        .spacing(10)
        .into()
}

fn view_object<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_object: &'a NodeObject,
) -> Element<'a, PageMsg> {
    fn append_data_path(data_path: &[DataPathType], name: &str) -> Vec<DataPathType> {
        let mut new_vec = Vec::with_capacity(data_path.len() + 1);
        new_vec.extend_from_slice(data_path);
        new_vec.push(DataPathType::Name(name.to_string()));
        new_vec
    }

    column()
        .push_maybe(
            node.desc
                .as_ref()
                .map(|desc| section().title("Description").add(text(desc))),
        )
        .push(
            section()
                .title("Values")
                .extend(node_object.nodes.iter().map(|(name, node)| {
                    mouse_area(
                        row().push(text(name)).push(horizontal_space()).push_maybe(
                            match &node.node {
                                Node::Null => Some(Element::from(text("null"))),
                                Node::Bool(node_bool) => Some(
                                    toggler(node_bool.value.unwrap_or_default())
                                        .on_toggle(move |value| {
                                            PageMsg::ChangeMsg(
                                                append_data_path(data_path, name),
                                                ChangeMsg::ChangeBool(value),
                                            )
                                        })
                                        .into(),
                                ),

                                Node::Enum(node_enum) => {
                                    #[derive(Eq, Clone)]
                                    struct Key<'a> {
                                        pub pos: usize,
                                        pub value: Cow<'a, str>,
                                    }

                                    impl PartialEq for Key<'_> {
                                        fn eq(&self, other: &Self) -> bool {
                                            self.pos == other.pos
                                        }
                                    }

                                    #[allow(clippy::to_string_trait_impl)]
                                    impl ToString for Key<'_> {
                                        fn to_string(&self) -> String {
                                            self.value.to_string()
                                        }
                                    }

                                    Some(
                                        row()
                                            .push_maybe(node_enum.value.map(|pos| {
                                                text(
                                                    node_enum.nodes[pos]
                                                        .name()
                                                        .unwrap_or(Cow::Owned(pos.to_string())),
                                                )
                                            }))
                                            .push(pick_list(
                                                node_enum
                                                    .nodes
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(pos, node)| Key {
                                                        pos,
                                                        value: node
                                                            .name()
                                                            .unwrap_or(Cow::Owned(pos.to_string())),
                                                    })
                                                    .collect::<Vec<_>>(),
                                                node_enum.value.map(|pos| Key {
                                                    pos,
                                                    value: Cow::Borrowed(""),
                                                }),
                                                |key| {
                                                    PageMsg::ChangeMsg(
                                                        append_data_path(data_path, name),
                                                        ChangeMsg::ChangeEnum(key.pos),
                                                    )
                                                },
                                            ))
                                            .align_y(alignment::Vertical::Center)
                                            .into(),
                                    )
                                }

                                _ => None,
                            },
                        ),
                    )
                    .on_press(PageMsg::OpenDataPath(DataPathType::Name(name.to_string())))
                })),
        )
        .push_maybe(node.default.as_ref().map(|default| {
            section().title("Default").add(
                row()
                    .push(horizontal_space())
                    .push(
                        // xxx: the on_press need to be lazy
                        button::text("reset to default").on_press(PageMsg::ChangeMsg(
                            data_path.to_vec(),
                            ChangeMsg::ApplyDefault,
                        )),
                    )
                    .push(tooltip(
                        icon!("report24"),
                        text("This will remove all children"),
                        Position::Top,
                    )),
            )
        }))
        .spacing(SPACING)
        .into()
}

fn no_value_defined_warning_icon<'a, M: 'a>() -> Element<'a, M> {
    tooltip(
        icon!("report24").class(cosmic::theme::Svg::custom(|e| cosmic::widget::svg::Style {
            color: Some(Color::from_rgb(236.0, 194.0, 58.0)),
        })),
        text("No value has been defined"),
        Position::Top,
    )
    .into()
}

fn view_bool<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_bool: &'a NodeBool,
) -> Element<'a, PageMsg> {
    column()
        .push_maybe(
            node.desc
                .as_ref()
                .map(|desc| section().title("Description").add(text(desc))),
        )
        .push(
            section().title("Value").add(
                row()
                    .push(text("Current value"))
                    .push(horizontal_space())
                    .push(
                        toggler(node_bool.value.unwrap_or_default()).on_toggle(move |value| {
                            PageMsg::ChangeMsg(data_path.to_vec(), ChangeMsg::ChangeBool(value))
                        }),
                    )
                    .push_maybe(if node_bool.value.is_none() {
                        Some(no_value_defined_warning_icon())
                    } else {
                        None
                    }),
            ),
        )
        .push_maybe(
            node.default
                .as_ref()
                .and_then(|v| v.to_bool())
                .map(|default| {
                    section()
                        .title("Default")
                        .add(
                            row()
                                .push(text("Default value"))
                                .push(horizontal_space())
                                .push(toggler(default)),
                        )
                        .add(row().push(horizontal_space()).push(
                            // xxx: the on_press need to be lazy
                            button::text("reset to default").on_press(PageMsg::ChangeMsg(
                                data_path.to_vec(),
                                ChangeMsg::ApplyDefault,
                            )),
                        ))
                }),
        )
        .spacing(SPACING)
        .into()
}

fn view_string<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_string: &'a NodeString,
) -> Element<'a, PageMsg> {
    column()
        .push_maybe(
            node.desc
                .as_ref()
                .map(|desc| section().title("Description").add(text(desc))),
        )
        .push(
            section().title("Value").add(
                row()
                    .push(text("Current value"))
                    .push(horizontal_space())
                    .push(
                        text_input("value", node_string.value.as_ref().map_or("", |v| v)).on_input(
                            move |value| {
                                PageMsg::ChangeMsg(
                                    data_path.to_vec(),
                                    ChangeMsg::ChangeString(value),
                                )
                            },
                        ),
                    )
                    .push_maybe(if node_string.value.is_none() {
                        Some(no_value_defined_warning_icon())
                    } else {
                        None
                    }),
            ),
        )
        .push_maybe(
            node.default
                .as_ref()
                .and_then(|v| v.as_str())
                .map(|default| {
                    section()
                        .title("Default")
                        .add(
                            row()
                                .push(text("Default value"))
                                .push(horizontal_space())
                                .push(text(default)),
                        )
                        .add(row().push(horizontal_space()).push(
                            // xxx: the on_press need to be lazy
                            button::text("reset to default").on_press(PageMsg::ChangeMsg(
                                data_path.to_vec(),
                                ChangeMsg::ApplyDefault,
                            )),
                        ))
                }),
        )
        .spacing(SPACING)
        .into()
}

fn view_number<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_number: &'a NodeNumber,
) -> Element<'a, PageMsg> {
    column()
        .push_maybe(
            node.desc
                .as_ref()
                .map(|desc| section().title("Description").add(text(desc))),
        )
        .push(
            section().title("Value").add(
                row()
                    .push(text("Current value"))
                    .push(horizontal_space())
                    .push(
                        text_input("value", &node_number.value_string).on_input(move |value| {
                            PageMsg::ChangeMsg(data_path.to_vec(), ChangeMsg::ChangeNumber(value))
                        }),
                    )
                    .push_maybe(if node_number.value.is_none() {
                        Some(no_value_defined_warning_icon())
                    } else if match node_number.kind {
                        NumberKind::Integer => node_number.value_string.parse::<i128>().is_err(),
                        NumberKind::Float => node_number.value_string.parse::<f64>().is_err(),
                    } {
                        Some(
                            tooltip(
                                icon!("report24"),
                                text("This value is incorrect."),
                                Position::Top,
                            )
                            .into(),
                        )
                    } else {
                        None
                    }),
            ),
        )
        .push_maybe(
            node.default
                .as_ref()
                .and_then(|v| v.to_num())
                .and_then(|v| node_number.parse_number(v))
                .map(|default| {
                    section()
                        .title("Default")
                        .add(
                            row()
                                .push(text("Default value"))
                                .push(horizontal_space())
                                .push(text(default.to_string())),
                        )
                        .add(row().push(horizontal_space()).push(
                            // xxx: the on_press need to be lazy
                            button::text("reset to default").on_press(PageMsg::ChangeMsg(
                                data_path.to_vec(),
                                ChangeMsg::ApplyDefault,
                            )),
                        ))
                }),
        )
        .spacing(SPACING)
        .into()
}

fn view_enum<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_enum: &'a NodeEnum,
) -> Element<'a, PageMsg> {
    let (value_pos, value) = node_enum.unwrap_value();

    column()
        .push_maybe(
            node.desc
                .as_ref()
                .map(|desc| section().title("Description").add(text(desc))),
        )
        .push(
            section()
                .title("Values")
                .extend(node_enum.nodes.iter().enumerate().map(|(pos, node)| {
                    container(cosmic::widget::radio(
                        row()
                            .push(text(node.name().unwrap_or(Cow::Owned(pos.to_string()))))
                            .push(horizontal_space())
                            .push_maybe(
                                if let Some(active_pos) = node_enum.value
                                    && active_pos == pos
                                {
                                    Some(
                                        button::text("modify").on_press(PageMsg::OpenDataPath(
                                            DataPathType::Indice(pos),
                                        )),
                                    )
                                } else {
                                    None
                                },
                            ),
                        pos,
                        node_enum.value,
                        |pos| PageMsg::ChangeMsg(data_path.to_vec(), ChangeMsg::ChangeEnum(pos)),
                    ))
                    .padding(5)
                })),
        )
        .push_maybe(node.default.as_ref().map(|default| {
            section()
                .title("Default")
                .add_maybe(default.clone().into_string().map(|default| {
                    container(
                        row()
                            .push(text("Default value"))
                            .push(horizontal_space())
                            .push(text(default)),
                    )
                    .padding(10)
                }))
                .add(
                    row()
                        .push(horizontal_space())
                        .push(
                            // xxx: the on_press need to be lazy
                            button::text("reset to default").on_press(PageMsg::ChangeMsg(
                                data_path.to_vec(),
                                ChangeMsg::ApplyDefault,
                            )),
                        )
                        .push(tooltip(
                            icon!("report24"),
                            text("This will remove all children"),
                            Position::Top,
                        )),
                )
        }))
        .spacing(SPACING)
        .into()
}

fn view_value<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_value: &'a NodeValue,
) -> Element<'a, PageMsg> {
    column()
        .push(text("i'm just a value"))
        .push(text(format!("name: {:?}", data_path.last())))
        .push(text(format!("{:?}", node_value.value)))
        .into()
}

fn view_array<'a>(
    data_path: &'a [DataPathType],
    node: &'a NodeContainer,
    node_array: &'a NodeArray,
) -> Element<'a, PageMsg> {
    let mut elements = Vec::new();

    for (pos, node) in node_array.values.iter().enumerate() {
        let element = button::text(format!("{}", pos))
            .on_press(PageMsg::OpenDataPath(DataPathType::Indice(pos)))
            .into();
        elements.push(element);
    }

    column::with_children(elements).into()
}
