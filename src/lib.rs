use mercy::alloc::AllocatesTypes;
use turubai::elements::{ChildrenFunction, Component, Element, Modifiers, Text};
use turubai_macros::{markup, markups, state};

#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
    unsafe { std::alloc::alloc(layout) }
}

mod js;

pub mod threads;

pub static BOX_URL: &str = "https://itsjustbox.com/";

pub struct BoxDisplayName {
    modifiers: Modifiers,
    memory_context: mercy::context::Context,
    renderer: turubai::runtime::Renderer,
}

impl BoxDisplayName {
    pub fn turubai_new_with_0_args(
        modifiers: Modifiers,
        _children: ChildrenFunction<BoxDisplayName>,
        memory_context: &mut mercy::context::Context,
        renderer: turubai::runtime::Renderer,
    ) -> Self {
        Self {
            modifiers,
            memory_context: memory_context.clone(),
            renderer,
        }
    }
}

impl Component for BoxDisplayName {
    #[markups]
    fn markup(&self) -> mercy::boxed::Box<dyn Element> {
        let mut memory_context = self.memory_context.clone();
        let username = js::get_our_username();

        let display_name = state!(memory_context, new_string(""));
        let mut display_name_state = display_name.clone();

        mercy::pal::wasm::spawn_local(async move {
            let name = js::fetch_display_name(&username).await;
            display_name_state.set_value(name);
        });

        let renderer = self.renderer.clone();
        markup! (renderer: renderer, memory_context: memory_context, {
            Text(display_name, modifiers: self.modifiers.clone())
        })
    }
}
