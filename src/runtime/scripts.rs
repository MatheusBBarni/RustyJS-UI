pub fn bootstrap() -> &'static str {
    r#"
class CallbackRegistry {
    constructor() {
        this.callbacks = new Map();
        this.nextId = 1;
    }

    register(fn) {
        if (typeof fn !== 'function') {
            return null;
        }

        const id = `cb_${this.nextId++}`;
        this.callbacks.set(id, fn);
        return id;
    }

    trigger(id, payload) {
        const fn = this.callbacks.get(id);
        if (!fn) {
            console.warn(`Callback ${id} not found.`);
            return;
        }

        fn(payload);
    }

    clear() {
        this.callbacks.clear();
    }
}

const GlobalCallbackRegistry = new CallbackRegistry();

globalThis.RustBridge = {
    trigger: (id, payload) => GlobalCallbackRegistry.trigger(id, payload)
};

if (!globalThis.console) {
    globalThis.console = {
        warn() {},
        error() {}
    };
}

globalThis.__SEND_TO_RUST__ = (payload) => {
    __RUSTYJS_NATIVE_CAPTURE__(String(payload));
};

function flattenChildren(target, value) {
    if (Array.isArray(value)) {
        for (const child of value) {
            flattenChildren(target, child);
        }
        return;
    }

    if (value !== undefined && value !== null) {
        target.push(value);
    }
}

function normalizeStyle(style = {}) {
    return {
        direction: style.direction ?? style.flexDirection,
        padding: style.padding,
        spacing: style.spacing,
        width: style.width,
        height: style.height,
        alignItems: style.alignItems,
        justifyContent: style.justifyContent,
        backgroundColor: style.backgroundColor,
        borderColor: style.borderColor,
        borderWidth: style.borderWidth,
        borderRadius: style.borderRadius,
        color: style.color,
        fontSize: style.fontSize,
        fontWeight: style.fontWeight
    };
}

function createNode(type, props = {}) {
    const node = { type, props: {}, children: [] };

    for (const [key, value] of Object.entries(props)) {
        if (key === 'children') {
            flattenChildren(node.children, value);
            continue;
        }

        if (key === 'style') {
            node.props.style = normalizeStyle(value);
            continue;
        }

        if (typeof value === 'function') {
            node.props[key] = GlobalCallbackRegistry.register(value);
            continue;
        }

        node.props[key] = value;
    }

    return node;
}

globalThis.View = (props) => createNode('View', props);
globalThis.Text = (props) => createNode('Text', props);
globalThis.Button = (props) => createNode('Button', props);
globalThis.TextInput = (props) => createNode('TextInput', props);

class AppEngine {
    constructor() {
        this.rootRenderFn = null;
        this.isRenderPending = false;
    }

    run(config) {
        this.rootRenderFn = config.render;

        __SEND_TO_RUST__(JSON.stringify({
            action: 'INIT_WINDOW',
            title: config.title || 'IcedJS App',
            width: config.windowSize?.width || 800,
            height: config.windowSize?.height || 600
        }));

        this.requestRender();
    }

    requestRender() {
        if (this.isRenderPending) {
            return;
        }

        this.isRenderPending = true;
        Promise.resolve().then(() => {
            try {
                this.executeRender();
            } catch (error) {
                console.error('RustyJS-UI render failed:', error);
                throw error;
            }
        });
    }

    executeRender() {
        try {
            GlobalCallbackRegistry.clear();

            const vdomTree = this.rootRenderFn();

            __SEND_TO_RUST__(JSON.stringify({
                action: 'UPDATE_VDOM',
                tree: vdomTree
            }));
        } finally {
            this.isRenderPending = false;
        }
    }
}

globalThis.App = new AppEngine();
"#
}

pub fn counter_app() -> &'static str {
    r#"
let counter = 0;

function increment() {
    counter += 1;
    App.requestRender();
}

function AppLayout() {
    return View({
        style: {
            direction: 'column',
            padding: 20,
            spacing: 12,
            alignItems: 'center',
            justifyContent: 'center'
        },
        children: [
            Text({
                text: `Count is: ${counter}`,
                style: {
                    fontSize: 24,
                    color: '#111111'
                }
            }),
            Button({
                text: 'Increment',
                onClick: increment,
                style: {
                    padding: 10,
                    backgroundColor: '#007AFF',
                    borderRadius: 8
                }
            })
        ]
    });
}

App.run({
    title: 'Counter App',
    windowSize: { width: 400, height: 300 },
    render: AppLayout
});
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_uses_string_callback_ids() {
        let source = bootstrap();
        assert!(source.contains("return id;"));
        assert!(!source.contains("return { id };"));
    }

    #[test]
    fn bootstrap_defends_pending_state_on_render_errors() {
        let source = bootstrap();
        assert!(source.contains("finally {"));
        assert!(source.contains("this.isRenderPending = false;"));
    }

    #[test]
    fn bundled_sample_app_matches_prd_shape() {
        let source = counter_app();
        assert!(source.contains("function AppLayout()"));
        assert!(source.contains("App.requestRender();"));
        assert!(source.contains("Button({"));
        assert!(source.contains("Text({"));
    }
}
