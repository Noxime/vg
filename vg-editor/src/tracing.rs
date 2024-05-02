use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};
use dashmap::DashSet;
use tracing::{
    field::{Field, FieldSet, Visit},
    metadata::Kind,
    Event, Level, Metadata, Subscriber,
};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{layer::Context, prelude::*, util::SubscriberInitExt, EnvFilter, Layer};

pub struct Tracing {
    list: Mutex<Vec<TracingEvent>>,
    string_cache: DashSet<&'static str>,
    fields_cache: DashSet<&'static [&'static str]>,
}

impl Tracing {
    pub fn new() -> Tracing {
        Tracing {
            list: Mutex::new(vec![]),
            string_cache: DashSet::new(),
            fields_cache: DashSet::new(),
        }
    }

    pub fn push(&self, event: TracingEvent) {
        let mut list = self.list.lock().unwrap();
        list.push(event);
    }

    pub fn len(&self) -> usize {
        self.list.lock().unwrap().len()
    }

    pub fn with(&self, i: usize, mut f: impl FnMut(&TracingEvent)) {
        let list = self.list.lock().unwrap();
        if let Some(event) = list.get(i) {
            f(event)
        }
    }

    pub fn clear(&self) {
        self.list.lock().unwrap().clear();
    }

    pub fn memoize(&self, s: &str) -> &'static str {
        if !self.string_cache.contains(s) {
            let mut alloc = s.to_string();
            alloc.shrink_to_fit(); // Ensure no memory is wasted
            let leaked = Box::leak(alloc.into_boxed_str());
            self.string_cache.insert(leaked);
        }
        *self.string_cache.get(s).unwrap()
    }

    pub fn memoize_fields(&self, fields: &FieldSet) -> &'static [&'static str] {
        let mut names = vec![];

        for name in fields.iter() {
            let name = self.memoize(name.name());
            names.push(name);
        }

        names.shrink_to_fit();

        if !self.fields_cache.contains(names.as_slice()) {
            self.fields_cache.insert(names.clone().leak());
        }

        *self.fields_cache.get(names.as_slice()).unwrap()
    }
}

pub struct LevelMask {
    pub trace: bool,
    pub debug: bool,
    pub info: bool,
    pub warn: bool,
    pub error: bool,
}

impl LevelMask {
    pub fn all() -> LevelMask {
        LevelMask {
            trace: true,
            debug: true,
            info: true,
            warn: true,
            error: true,
        }
    }
}

pub struct TracingEvent {
    pub time: DateTime<Local>,
    pub metadata: &'static Metadata<'static>,
    pub values: Vec<String>,
}

impl TracingEvent {
    pub fn fields(&self) -> impl Iterator<Item = (&'static str, &str)> + '_ {
        self.metadata
            .fields()
            .iter()
            .map(|f| f.name())
            .zip(self.values.iter().map(|s| s.as_str()))
    }
}

pub fn init() -> Arc<Tracing> {
    let this = Arc::new(Tracing::new());

    tracing_subscriber::registry()
        .with(EditorLayer::new(Arc::clone(&this)))
        .with(tracing_subscriber::fmt::Layer::new().with_filter(EnvFilter::from_default_env()))
        .init();

    this
}

pub struct EditorLayer {
    tracing: Arc<Tracing>,
}

impl EditorLayer {
    pub fn new(tracing: Arc<Tracing>) -> EditorLayer {
        EditorLayer { tracing }
    }
}

impl<S: Subscriber> Layer<S> for EditorLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Filter non-vg messages above Info level
        if !event.metadata().target().starts_with("vg_") {
            if *event.metadata().level() > Level::INFO {
                return;
            }
        }

        let time = Local::now();

        let metadata = if let Some(m) = event.normalized_metadata() {
            let new = Metadata::new(
                self.tracing.memoize(m.name()),
                self.tracing.memoize(m.target()),
                *m.level(),
                m.file().map(|s| self.tracing.memoize(s)),
                m.line(),
                m.module_path().map(|s| self.tracing.memoize(s)),
                FieldSet::new(self.tracing.memoize_fields(m.fields()), m.callsite()),
                Kind::EVENT,
            );

            // TODO: Memoize full metadata, naive impl needs Hash
            Box::leak(Box::new(new))
        } else {
            event.metadata()
        };

        struct Visitor<'a>(&'a mut Vec<String>);

        impl<'a> Visit for Visitor<'a> {
            fn record_debug(&mut self, _: &Field, value: &dyn std::fmt::Debug) {
                self.0.push(format!("{value:?}"));
            }
        }

        let mut values = vec![];
        event.record(&mut Visitor(&mut values));

        self.tracing.push(TracingEvent {
            time,
            metadata,
            values,
        });
    }
}
