use turubai::{
    Unit::Em,
    composition::{HStack, HorizontalAlignment, Spacer, VStack},
    control_flow::ForEach,
    elements::{
        ChildrenFunction, Element, Modifiers, Text, TextDecoration, TextDecorationLine,
        TextLineStyle,
    },
    font::{Font, FontWeight},
    postprocessing::{frame, padding},
    shadow::ShadowDescriptor,
};

use mercy::alloc::AllocatesTypes;
use turubai_macros::{markup, state, string};

use crate::js::{self, format_time};

#[derive(Clone, Debug)]
pub struct Thread {
    username: String,
    title: String,
    message: String,
    time: usize,
}

impl Thread {
    pub fn new(username: &str, title: &str, message: &str, time: usize) -> Self {
        Self {
            username: String::from(username),
            title: String::from(title),
            message: String::from(message),
            time,
        }
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }

    pub fn time(&self) -> usize {
        self.time
    }

    pub async fn receive_users_threads(username: &str, offset: usize, size: usize) -> Vec<Thread> {
        js::fetch_threads(username, offset, size).await
    }
}

pub struct BoxThreads {
    element: mercy::boxed::Box<dyn Element>,
}

impl BoxThreads {
    pub fn turubai_new_with_2_args(
        offset_index: mercy::rec::State<mercy::boxed::Box<usize>>,
        size_index: mercy::rec::State<mercy::boxed::Box<usize>>,
        modifiers: Modifiers,
        _children: ChildrenFunction<BoxThreads>,
        memory_context: &mut mercy::context::Context,
        renderer: turubai::runtime::Renderer,
    ) -> Self {
        let threads = state!(memory_context, new_vec());

        let mut threads_state_clone = threads.clone();
        let offset_index_clone = offset_index.clone();
        let size_index_clone = size_index.clone();
        mercy::pal::wasm::spawn_local(async move {
            let username = js::get_our_username();
            let fetched = js::fetch_threads(
                &username,
                offset_index_clone.value(),
                size_index_clone.value(),
            )
            .await;
            threads_state_clone.set_value(fetched);
        });

        let mut threads_state_clone = threads.clone();
        let offset_index_clone = offset_index.clone();
        let size_index_clone = size_index.clone();
        offset_index.clone().add_listener_callback(move |_| {
            let offset_index_clone = offset_index_clone.clone();
            let size_index_clone = size_index_clone.clone();
            let mut threads_state_clone = threads_state_clone.clone();
            mercy::pal::wasm::spawn_local(async move {
                let username = js::get_our_username();
                let fetched = js::fetch_threads(
                    &username,
                    offset_index_clone.value(),
                    size_index_clone.value(),
                )
                .await;
                threads_state_clone.set_value(fetched);
            });
        });

        let mut threads_state_clone = threads.clone();
        let offset_index_clone = offset_index.clone();
        let size_index_clone = size_index.clone();
        size_index.clone().add_listener_callback(move |_| {
            let offset_index_clone = offset_index_clone.clone();
            let size_index_clone = size_index_clone.clone();
            let mut threads_state_clone = threads_state_clone.clone();
            mercy::pal::wasm::spawn_local(async move {
                let username = js::get_our_username();
                let fetched = js::fetch_threads(
                    &username,
                    offset_index_clone.value(),
                    size_index_clone.value(),
                )
                .await;
                threads_state_clone.set_value(fetched);
            });
        });

        let renderer_for_listener = renderer.clone();
        threads
            .clone()
            .add_listener_callback(move |_| renderer_for_listener.redo_rendering());

        let element = Self::build_element(modifiers, memory_context, renderer, threads);

        Self { element }
    }

    fn build_element(
        modifiers: Modifiers,
        memory_context: &mut mercy::context::Context,
        renderer: turubai::runtime::Renderer,
        threads: mercy::rec::State<mercy::vec::Vec<Thread>>,
    ) -> mercy::boxed::Box<dyn Element> {
        let subheader_font = Font::new(memory_context, "Inter", 10, FontWeight::Bold, false);
        let text_font = Font::new(memory_context, "Inter", 11, FontWeight::Regular, false);
        let bold_text_font = Font::new(memory_context, "Inter", 11, FontWeight::Bold, false);

        let text_decoration = TextDecoration {
            underline: TextDecorationLine {
                style: TextLineStyle::Single,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut memory_context = memory_context.clone();

        markup! (renderer: renderer, memory_context: memory_context, {
            VStack(modifiers: modifiers) {
                for thread in threads.lock().iter() {
                    HStack(spacing: Em(0.5)) {
                        Text(string!(memory_context, &thread.username()), decoration: text_decoration, font: text_font.clone())
                        Spacer
                        VStack(spacing: Em(0.5), alignment: HorizontalAlignment::Leading) {
                            Text(string!(memory_context, &thread.title()), font: subheader_font)
                            Text(string!(memory_context, &thread.message()), font: text_font.clone())
                            HStack(spacing: Em(0.25)) {
                                Text("Posted", font: text_font)
                                Text(string!(memory_context, &format_time(thread.time())), font: bold_text_font)
                            }
                        }
                        .frame(width: Em(20.0))
                    }
                    .padding(all: Em(1.0))
                    .frame(width: Em(24.0))
                }
            }
        })
    }

    fn markup(&self) -> &dyn Element {
        self.element.as_ref()
    }
}

impl Element for BoxThreads {
    fn name(&self) -> &'static str {
        "component"
    }

    fn display_name(&self) -> &'static str {
        "BoxThreads"
    }

    fn shadow_descriptor(&self) -> ShadowDescriptor {
        ShadowDescriptor::view()
    }

    fn child_count(&self) -> usize {
        1
    }

    fn for_each_child(&self, f: &mut dyn FnMut(&dyn Element)) {
        f(self.markup());
    }

    fn internal_structure_changed(&self) {
        self.element.internal_structure_changed();
    }
}
