use anyhow::Result;
use gst::prelude::*;
use gst::glib;
use gst::glib::once_cell::sync::Lazy;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "main",
        gst::DebugColorFlags::empty(),
        Some("Main function"),
    )
});

fn main() -> Result<()> {
    gst::init()?;

    let pipeline = gst::parse_launch(r#"

    videotestsrc ! videoconvert ! timeoverlay shaded-background=true ! gtksink

    "#,)?.downcast::<gst::Pipeline>().unwrap();

    let context = glib::MainContext::default();
    let main_loop = glib::MainLoop::new(Some(&context), false);

    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline.bus().unwrap();
    bus.add_watch({
        let main_loop = main_loop.clone();
        move |_, msg| {
            use gst::MessageView;
            let main_loop = &main_loop;
            match msg.view() {
                MessageView::Eos(..) => main_loop.quit(),
                MessageView::Error(err) => {
                    gst::error!(CAT, obj: &err.src().unwrap(),
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    main_loop.quit();
                }
                _ => (),
            };
            glib::Continue(true)
        }
    })
        .expect("Failed to add bus watch");

    main_loop.run();

    pipeline.set_state(gst::State::Null)?;
    bus.remove_watch().unwrap();

    Ok(())
}
