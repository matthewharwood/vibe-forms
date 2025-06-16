use maud::{html, Markup};
use serde::Serialize;
use form_macro::FormGen;


pub trait FormComponent {
    fn render(&self, field_name: &str) -> Markup;
}
#[derive(Clone, Debug, Serialize, FormGen)]
pub struct TextField {
    #[mark]
    pub value: String,
    pub label: String,
    pub name: Option<String>,
    pub id: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: bool,
    pub readonly: bool,
    pub class: Option<String>,
}
impl FormComponent for TextField {
    fn render(&self, field_name: &str) -> Markup {
        let name_attr = format!("{}_value", field_name);
        html! {
            div class="form-field" {
                @if !self.label.is_empty() {
                    label for=(self.id.as_deref().unwrap_or(&name_attr)) { 
                        (self.label) 
                    }
                }
                input type="text" 
                   id=(self.id.as_deref().unwrap_or(&name_attr))
                   name=(name_attr)
                   value=(self.value)
                   placeholder=[self.placeholder.as_deref()]
                   class=[self.class.as_deref()]
                   disabled[self.disabled]
                   readonly[self.readonly] {}
            }
        }
    }
}

impl Default for TextField {
    fn default() -> Self {
        Self {
            value: String::new(),
            label: String::new(),
            name: None,
            id: None,
            placeholder: None,
            disabled: false,
            readonly: false,
            class: None,
        }
    }
}
// TextArea field type for m ulti-line text input
#[derive(Clone, Debug, Serialize, FormGen)]
pub struct TextAreaField {
    #[mark]
    pub value: String,
    pub label: String,
    pub name: Option<String>,
    pub id: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: bool,
    pub readonly: bool,
    pub class: Option<String>,
    pub rows: Option<u32>,
    pub cols: Option<u32>,
}

impl FormComponent for TextAreaField {
    fn render(&self, field_name: &str) -> Markup {
        let name_attr = format!("{}_value", field_name);
        html! {
            div class="form-field" {
                @if !self.label.is_empty() {
                    label for=(self.id.as_deref().unwrap_or(&name_attr)) {
                        (self.label)
                    }
                }
                textarea id=(self.id.as_deref().unwrap_or(&name_attr))
                         name=(name_attr)
                         placeholder=[self.placeholder.as_deref()]
                         class=[self.class.as_deref()]
                         rows=[self.rows.map(|r| r.to_string()).as_deref()]
                         cols=[self.cols.map(|c| c.to_string()).as_deref()]
                         disabled[self.disabled]
                         readonly[self.readonly] {
                    (self.value)
                }
            }
        }
    }
}
impl Default for TextAreaField {
    fn default() -> Self {
        Self {
            value: String::new(),
            label: String::new(),
            name: None,
            id: None,
            placeholder: None,
            disabled: false,
            readonly: false,
            class: None,
            rows: Some(4),
            cols: None,
        }
    }
}