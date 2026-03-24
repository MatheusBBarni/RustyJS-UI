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

class PendingFetchRegistry {
    constructor() {
        this.requests = new Map();
        this.nextId = 1;
    }

    register(record) {
        const id = `fetch_${this.nextId++}`;
        this.requests.set(id, record);
        return id;
    }

    trigger(id, handler) {
        const record = this.requests.get(id);
        if (!record) {
            return false;
        }

        this.requests.delete(id);
        handler(record);
        return true;
    }
}

const GlobalCallbackRegistry = new CallbackRegistry();
const PendingFetchRegistryInstance = new PendingFetchRegistry();

function normalizeFetchHeaders(headers = {}) {
    if (!headers) {
        return {};
    }

    if (typeof Headers !== 'undefined' && headers instanceof Headers) {
        const normalized = {};

        headers.forEach((value, key) => {
            normalized[String(key)] = String(value);
        });

        return normalized;
    }

    if (headers instanceof Map) {
        const normalized = {};

        for (const [key, value] of headers.entries()) {
            normalized[String(key)] = String(value);
        }

        return normalized;
    }

    if (Array.isArray(headers)) {
        const normalized = {};

        for (const entry of headers) {
            if (!Array.isArray(entry) || entry.length < 2) {
                continue;
            }

            const [key, value] = entry;
            normalized[String(key)] = String(value);
        }

        return normalized;
    }

    if (typeof headers === 'object') {
        const normalized = {};

        for (const [key, value] of Object.entries(headers)) {
            if (value === undefined || value === null) {
                continue;
            }

            normalized[String(key)] = String(value);
        }

        return normalized;
    }

    return {};
}

function normalizeFetchBody(body, headers) {
    if (body === undefined || body === null) {
        return undefined;
    }

    if (typeof body === 'string') {
        return body;
    }

    if (typeof body === 'object') {
        if (!Object.keys(headers).some((key) => key.toLowerCase() === 'content-type')) {
            headers['Content-Type'] = 'application/json';
        }

        return JSON.stringify(body);
    }

    return String(body);
}

function normalizeFetchError(error) {
    if (error instanceof Error) {
        return error;
    }

    if (typeof error === 'string') {
        return new Error(error);
    }

    if (error && typeof error === 'object') {
        const message = error.message ?? error.error ?? JSON.stringify(error);
        const normalized = new Error(String(message));

        if (error.name) {
            normalized.name = String(error.name);
        }

        return normalized;
    }

    return new Error('Fetch request failed.');
}

function resolvePendingFetch(requestId, value) {
    const handled = PendingFetchRegistryInstance.trigger(requestId, (record) =>
        record.resolve(value)
    );

    if (!handled) {
        console.warn(`Fetch request ${requestId} not found.`);
    }
}

function rejectPendingFetch(requestId, error) {
    const handled = PendingFetchRegistryInstance.trigger(requestId, (record) =>
        record.reject(normalizeFetchError(error))
    );

    if (!handled) {
        console.warn(`Fetch request ${requestId} not found.`);
    }
}

globalThis.RustBridge = {
    trigger: (id, payload) => GlobalCallbackRegistry.trigger(id, payload),
    resolveFetch: resolvePendingFetch,
    rejectFetch: rejectPendingFetch
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

function dispatchToRust(payload) {
    __SEND_TO_RUST__(JSON.stringify(payload));
}

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

function appendRenderable(target, value, context) {
    if (typeof value === 'function') {
        appendRenderable(target, value(context), context);
        return;
    }

    flattenChildren(target, value);
}

function normalizeFlexValue(value) {
    if (typeof value !== 'string') {
        return value;
    }

    switch (value.trim().toLowerCase()) {
        case 'flex-start':
            return 'start';
        case 'flex-end':
            return 'end';
        case 'space-between':
            return 'space_between';
        case 'space-around':
            return 'space_around';
        case 'space-evenly':
            return 'space_evenly';
        default:
            return value;
    }
}

function normalizeStyle(style = {}) {
    return {
        direction: style.direction ?? style.flexDirection,
        padding: style.padding,
        spacing: style.spacing ?? style.gap,
        width: style.width,
        height: style.height,
        alignItems: normalizeFlexValue(style.alignItems),
        justifyContent: normalizeFlexValue(style.justifyContent),
        backgroundColor: style.backgroundColor,
        borderColor: style.borderColor,
        borderWidth: style.borderWidth,
        borderRadius: style.borderRadius,
        color: style.color,
        fontSize: style.fontSize,
        fontWeight: style.fontWeight
    };
}

function createFlatList(props = {}) {
    const {
        data = [],
        renderItem,
        keyExtractor: _keyExtractor,
        horizontal = false,
        style,
        contentContainerStyle,
        ListHeaderComponent,
        ListFooterComponent,
        ListEmptyComponent,
        ItemSeparatorComponent,
        children: _children,
        ...rest
    } = props;
    const items = Array.isArray(data) ? data : [];
    const listChildren = [];

    appendRenderable(listChildren, ListHeaderComponent, { data: items });

    if (items.length === 0) {
        appendRenderable(listChildren, ListEmptyComponent, { data: items });
    } else if (typeof renderItem === 'function') {
        for (let index = 0; index < items.length; index += 1) {
            const item = items[index];
            appendRenderable(listChildren, renderItem({ item, index }));

            if (index < items.length - 1) {
                appendRenderable(listChildren, ItemSeparatorComponent, {
                    leadingItem: item,
                    leadingIndex: index
                });
            }
        }
    } else {
        console.warn('FlatList requires a renderItem function.');
    }

    appendRenderable(listChildren, ListFooterComponent, { data: items });

    return createNode('FlatList', {
        ...rest,
        style: resolveFlatListStyle(style, horizontal),
        children: [
            View({
                style: resolveFlatListContentStyle(contentContainerStyle, horizontal),
                children: listChildren
            })
        ]
    });
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

function resolveFlatListStyle(style, horizontal) {
    const direction = horizontal ? 'row' : 'column';
    const baseStyle = style || {};

    return {
        ...baseStyle,
        width: baseStyle.width ?? 'fill',
        height: baseStyle.height ?? 'fill',
        direction,
        flexDirection: direction
    };
}

function resolveFlatListContentStyle(style, horizontal) {
    const direction = horizontal ? 'row' : 'column';
    const baseStyle = style || {};

    return {
        ...baseStyle,
        width: baseStyle.width ?? (horizontal ? 'auto' : 'fill'),
        height: baseStyle.height ?? (horizontal ? 'fill' : 'auto'),
        direction,
        flexDirection: direction
    };
}

globalThis.View = (props) => createNode('View', props);
globalThis.Text = (props) => createNode('Text', props);
globalThis.Button = (props) => createNode('Button', props);
globalThis.TextInput = (props) => createNode('TextInput', props);
globalThis.SelectInput = (props) => createNode('SelectInput', props);
globalThis.FlatList = (props) => createFlatList(props);
globalThis.Modal = (props) => createNode('Modal', props);

globalThis.fetch = (url, options = {}) => new Promise((resolve, reject) => {
    const requestId = PendingFetchRegistryInstance.register({
        resolve,
        reject
    });

    const normalizedHeaders = normalizeFetchHeaders(options.headers);
    const normalizedBody = normalizeFetchBody(options.body, normalizedHeaders);
    const method = String(options.method || 'GET').toUpperCase();

    dispatchToRust({
        action: 'FETCH_REQUEST',
        requestId,
        url: String(url),
        method,
        headers: normalizedHeaders,
        ...(normalizedBody === undefined ? {} : { body: normalizedBody })
    });
});

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
