class CallbackRegistry {
    constructor() {
        this.callbacks = new Map();
        this.generations = [];
        this.currentGeneration = null;
        this.nextId = 1;
    }

    beginRender() {
        const generation = {
            id: this.nextId,
            callbacks: []
        };

        this.currentGeneration = generation;
        this.generations.push(generation);
    }

    register(fn) {
        if (typeof fn !== 'function') {
            return null;
        }

        const id = `cb_${this.nextId++}`;
        this.callbacks.set(id, fn);

        if (this.currentGeneration) {
            this.currentGeneration.callbacks.push(id);
        }

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

    // Native input can still deliver events from an older widget generation
    // while the app is already rendering newer controlled values.
    clearStale(maxGenerations = 256) {
        while (this.generations.length > maxGenerations) {
            const staleGeneration = this.generations.shift();

            if (!staleGeneration) {
                return;
            }

            for (const id of staleGeneration.callbacks) {
                this.callbacks.delete(id);
            }
        }
    }
}

class PendingAsyncRegistry {
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
const PendingFetchRegistryInstance = new PendingAsyncRegistry();
const PendingStorageRegistryInstance = new PendingAsyncRegistry();
const PendingTimerRegistryInstance = new PendingAsyncRegistry();
const AlertState = {
    current: null
};
const ToastState = {
    items: [],
    nextId: 1
};
const DevWarningRegistry = new Set();

function emitDevWarning(message, details) {
    dispatchToRust({
        action: 'DEV_WARNING',
        message,
        ...(details ? { details: String(details) } : {})
    });
}

function warnOnce(message, details) {
    if (DevWarningRegistry.has(message)) {
        return;
    }

    DevWarningRegistry.add(message);
    console.warn(message);
    emitDevWarning(message, details);
}

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

function resolvePendingStorage(requestId, value) {
    const handled = PendingStorageRegistryInstance.trigger(requestId, (record) =>
        record.resolve(value ?? null)
    );

    if (!handled) {
        console.warn(`Storage request ${requestId} not found.`);
    }
}

function rejectPendingStorage(requestId, error) {
    const handled = PendingStorageRegistryInstance.trigger(requestId, (record) =>
        record.reject(normalizeFetchError(error))
    );

    if (!handled) {
        console.warn(`Storage request ${requestId} not found.`);
    }
}

function resolvePendingTimer(requestId) {
    const handled = PendingTimerRegistryInstance.trigger(requestId, (record) =>
        record.resolve()
    );

    if (!handled) {
        console.warn(`Timer request ${requestId} not found.`);
    }
}

function rejectPendingTimer(requestId, error) {
    const handled = PendingTimerRegistryInstance.trigger(requestId, (record) =>
        record.reject(normalizeFetchError(error))
    );

    if (!handled) {
        console.warn(`Timer request ${requestId} not found.`);
    }
}

globalThis.RustBridge = {
    trigger: (id, payload) => GlobalCallbackRegistry.trigger(id, payload),
    resolveFetch: resolvePendingFetch,
    rejectFetch: rejectPendingFetch,
    resolveStorage: resolvePendingStorage,
    rejectStorage: rejectPendingStorage,
    resolveTimer: resolvePendingTimer,
    rejectTimer: rejectPendingTimer
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

function normalizeNodeProps(type, props = {}) {
    const normalized = { ...props };

    if (type === 'Button') {
        if (normalized.onPress !== undefined && normalized.onClick === undefined) {
            normalized.onClick = normalized.onPress;
        }
    }

    if (type === 'TextInput') {
        if (normalized.onValueChange !== undefined && normalized.onChange === undefined) {
            normalized.onChange = normalized.onValueChange;
        }

        if (normalized.onChange && !Object.prototype.hasOwnProperty.call(normalized, 'value')) {
            warnOnce('TextInput is controlled-only in this phase. Provide `value` alongside `onChange`.');
            normalized.value = '';
        }

        if (normalized.multiline) {
            warnOnce('TextInput.multiline is not implemented natively yet and will render as a single-line input.');
        }
    }

    if (type === 'SelectInput' || type === 'NativeSelect') {
        if (normalized.onValueChange !== undefined && normalized.onChange === undefined) {
            normalized.onChange = normalized.onValueChange;
        }

        if (normalized.onChange && !Object.prototype.hasOwnProperty.call(normalized, 'value')) {
            warnOnce(`${type} is controlled-only in this phase. Provide \`value\` alongside \`onChange\`.`);
            normalized.value = '';
        }
    }

    if (type === 'Modal') {
        if (normalized.onClose !== undefined && normalized.onRequestClose === undefined) {
            normalized.onRequestClose = normalized.onClose;
        }

        if (normalized.closeOnEscape === undefined) {
            normalized.closeOnEscape = true;
        }

        if (normalized.closeOnBackdrop === undefined) {
            normalized.closeOnBackdrop = false;
        }
    }

    return normalized;
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

function createNativeList(props = {}) {
    if (typeof props.onVisibleRangeChange === 'function') {
        warnOnce('NativeList.onVisibleRangeChange is not implemented natively yet. The callback will not fire in this phase.');
    }

    if (props.estimatedItemSize !== undefined) {
        warnOnce('NativeList virtualization is not implemented natively yet. estimatedItemSize is accepted for API compatibility only.');
    }

    return createFlatList(props);
}

function createNativeCombobox(props = {}) {
    const {
        value = '',
        options = [],
        placeholder = '',
        onChange,
        onInputChange,
        allowCustomValue = false,
        style,
        inputStyle,
        listStyle
    } = props;
    const normalizedValue = String(value ?? '');
    const normalizedOptions = Array.isArray(options) ? options : [];
    const filteredOptions = normalizedOptions.filter((option) => {
        const label = typeof option === 'string' ? option : option?.label ?? option?.value ?? '';
        const candidate = typeof label === 'string' ? label : String(label);
        return !normalizedValue || candidate.toLowerCase().includes(normalizedValue.toLowerCase());
    });
    const selectedOption = filteredOptions.find((option) => {
        const optionValue = typeof option === 'string' ? option : option?.value ?? option?.label ?? '';
        return String(optionValue) === normalizedValue;
    });

    return View({
        style: {
            direction: 'column',
            gap: 8,
            ...style
        },
        children: [
            TextInput({
                value: normalizedValue,
                placeholder,
                onChange: (nextValue) => {
                    if (typeof onInputChange === 'function') {
                        onInputChange(nextValue);
                    }

                    if (allowCustomValue && typeof onChange === 'function') {
                        onChange(nextValue);
                    }
                },
                style: inputStyle
            }),
            SelectInput({
                value: selectedOption ? normalizedValue : '',
                placeholder: filteredOptions.length === 0 ? 'No matching options' : 'Choose an option',
                options: filteredOptions,
                onChange,
                style: listStyle
            })
        ]
    });
}

function createTabs(props = {}) {
    const {
        value,
        tabs = [],
        onChange,
        style,
        tabListStyle,
        tabStyle,
        activeTabStyle,
        panelStyle
    } = props;
    const normalizedTabs = Array.isArray(tabs) ? tabs : [];
    const activeTab = normalizedTabs.find((tab) => tab && tab.value === value) ?? normalizedTabs[0] ?? null;
    const panelContent =
        activeTab && typeof activeTab.render === 'function'
            ? activeTab.render(activeTab)
            : activeTab?.content ?? null;

    return View({
        style: {
            direction: 'column',
            gap: 12,
            ...style
        },
        children: [
            View({
                style: {
                    direction: 'row',
                    gap: 8,
                    ...tabListStyle
                },
                children: normalizedTabs.map((tab) =>
                    Button({
                        text: String(tab?.label ?? tab?.value ?? ''),
                        disabled: Boolean(tab?.disabled),
                        onClick:
                            typeof onChange === 'function' && !tab?.disabled
                                ? () => onChange(tab.value)
                                : undefined,
                        style: {
                            padding: { x: 12, y: 10 },
                            borderRadius: 10,
                            backgroundColor:
                                activeTab && tab?.value === activeTab.value ? '#1E7A5F' : '#DDE6EF',
                            color:
                                activeTab && tab?.value === activeTab.value ? '#FFFFFF' : '#102033',
                            ...(tabStyle || {}),
                            ...(activeTab && tab?.value === activeTab.value ? activeTabStyle || {} : {})
                        }
                    })
                )
            }),
            View({
                style: {
                    direction: 'column',
                    ...panelStyle
                },
                children: [panelContent]
            })
        ]
    });
}

function createAlert(props = {}) {
    const {
        title = '',
        description = '',
        primaryButtonText = 'OK',
        primaryButtonOnClick,
        secondaryButtonText = 'Cancel',
        secondaryButtonOnClick,
        style,
        titleStyle,
        descriptionStyle,
        buttonContainerStyle,
        primaryButtonStyle,
        secondaryButtonStyle
    } = props;

    return View({
        style: {
            direction: 'column',
            gap: 12,
            padding: 16,
            borderWidth: 1,
            borderColor: '#D6D6D6',
            borderRadius: 10,
            ...style
        },
        children: [
            Text({
                text: String(title),
                style: {
                    fontSize: 20,
                    ...titleStyle
                }
            }),
            Text({
                text: String(description),
                style: {
                    color: '#5A5A5A',
                    ...descriptionStyle
                }
            }),
            View({
                style: {
                    direction: 'row',
                    gap: 10,
                    justifyContent: 'flex-end',
                    ...buttonContainerStyle
                },
                children: [
                    Button({
                        text: String(secondaryButtonText),
                        onClick: secondaryButtonOnClick,
                        style: {
                            padding: 10,
                            ...secondaryButtonStyle
                        }
                    }),
                    Button({
                        text: String(primaryButtonText),
                        onClick: primaryButtonOnClick,
                        style: {
                            padding: 10,
                            ...primaryButtonStyle
                        }
                    })
                ]
            })
        ]
    });
}

function dismissAlert() {
    AlertState.current = null;
}

function dismissToast(id) {
    const normalizedId = String(id ?? '');
    const nextItems = ToastState.items.filter((item) => item.id !== normalizedId);

    if (nextItems.length === ToastState.items.length) {
        return;
    }

    ToastState.items = nextItems;

    if (globalThis.App && typeof globalThis.App.requestRender === 'function') {
        globalThis.App.requestRender();
    }
}

function clearToasts() {
    if (ToastState.items.length === 0) {
        return;
    }

    ToastState.items = [];

    if (globalThis.App && typeof globalThis.App.requestRender === 'function') {
        globalThis.App.requestRender();
    }
}

function triggerToast(props = {}) {
    const id = `toast_${ToastState.nextId++}`;
    const item = {
        id,
        tone: props.tone ?? 'info',
        message: String(props.message ?? props.title ?? ''),
        actionLabel: props.actionLabel,
        onAction: props.onAction,
        onDismiss: props.onDismiss
    };

    ToastState.items = [...ToastState.items, item];

    const durationMs = Number(props.durationMs ?? 0);
    if (Number.isFinite(durationMs) && durationMs > 0) {
        Timer.after(durationMs)
            .then(() => {
                dismissToast(id);
                if (typeof item.onDismiss === 'function') {
                    item.onDismiss();
                }
            })
            .catch(() => {});
    }

    if (globalThis.App && typeof globalThis.App.requestRender === 'function') {
        globalThis.App.requestRender();
    }

    return id;
}

function renderActiveToasts() {
    if (ToastState.items.length === 0) {
        return null;
    }

    return View({
        style: {
            direction: 'column',
            gap: 10,
            alignItems: 'end'
        },
        children: ToastState.items.map((item) =>
            View({
                style: {
                    width: 320,
                    padding: { x: 14, y: 12 },
                    direction: 'column',
                    gap: 10,
                    borderRadius: 14,
                    borderWidth: 1,
                    borderColor: '#D5DEE8',
                    backgroundColor:
                        item.tone === 'success'
                            ? '#E7F6EF'
                            : item.tone === 'error'
                                ? '#FCE8E6'
                                : '#F7FAFC'
                },
                children: [
                    Text({
                        text: item.message,
                        style: {
                            color: '#102033'
                        }
                    }),
                    View({
                        style: {
                            direction: 'row',
                            gap: 10,
                            justifyContent: 'flex-end'
                        },
                        children: [
                            item.actionLabel
                                ? Button({
                                    text: String(item.actionLabel),
                                    onClick: () => {
                                        if (typeof item.onAction === 'function') {
                                            item.onAction();
                                        }
                                        dismissToast(item.id);
                                    },
                                    style: {
                                        padding: { x: 12, y: 10 },
                                        borderRadius: 10,
                                        backgroundColor: '#1E7A5F',
                                        color: '#FFFFFF'
                                    }
                                })
                                : null,
                            Button({
                                text: 'Dismiss',
                                onClick: () => {
                                    dismissToast(item.id);
                                    if (typeof item.onDismiss === 'function') {
                                        item.onDismiss();
                                    }
                                },
                                style: {
                                    padding: { x: 12, y: 10 },
                                    borderRadius: 10,
                                    backgroundColor: '#DDE6EF',
                                    color: '#102033'
                                }
                            })
                        ]
                    })
                ]
            })
        )
    });
}

function triggerAlert(props = {}) {
    AlertState.current = props;

    if (globalThis.App && typeof globalThis.App.requestRender === 'function') {
        globalThis.App.requestRender();
    }
}

function renderActiveAlert() {
    if (!AlertState.current) {
        return null;
    }

    const config = AlertState.current;

    const onClose = () => {
        dismissAlert();
        if (typeof config.onClose === 'function') {
            config.onClose();
        }
        globalThis.App.requestRender();
    };

    const secondaryButtonOnClick = () => {
        dismissAlert();
        if (typeof config.secondaryButtonOnClick === 'function') {
            config.secondaryButtonOnClick();
        }
        globalThis.App.requestRender();
    };

    const primaryButtonOnClick = () => {
        dismissAlert();
        if (typeof config.primaryButtonOnClick === 'function') {
            config.primaryButtonOnClick();
        }
        globalThis.App.requestRender();
    };

    return Modal({
        visible: true,
        onClose,
        children: [
            createAlert({
                ...config,
                secondaryButtonOnClick,
                primaryButtonOnClick
            })
        ]
    });
}

function createNode(type, props = {}) {
    props = normalizeNodeProps(type, props);
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

function normalizePathname(value) {
    let pathname = String(value ?? '').trim();

    if (!pathname) {
        return '/';
    }

    if (!pathname.startsWith('/')) {
        pathname = `/${pathname}`;
    }

    pathname = pathname.replace(/\/{2,}/g, '/');

    if (pathname.length > 1) {
        pathname = pathname.replace(/\/+$/, '');
    }

    return pathname || '/';
}

function safeDecodeURIComponent(value) {
    const normalized = String(value ?? '').replace(/\+/g, ' ');

    try {
        return decodeURIComponent(normalized);
    } catch (_error) {
        return normalized;
    }
}

function normalizePath(value) {
    const rawValue = String(value ?? '').trim();

    if (!rawValue) {
        return '/';
    }

    const hashIndex = rawValue.indexOf('#');
    const withoutHash = hashIndex >= 0 ? rawValue.slice(0, hashIndex) : rawValue;
    const queryIndex = withoutHash.indexOf('?');

    if (queryIndex < 0) {
        return normalizePathname(withoutHash);
    }

    const pathname = normalizePathname(withoutHash.slice(0, queryIndex));
    const search = withoutHash.slice(queryIndex + 1);

    return search ? `${pathname}?${search}` : pathname;
}

function parseQuery(search) {
    const query = {};
    const rawSearch = String(search ?? '').replace(/^\?/, '');

    if (!rawSearch) {
        return query;
    }

    for (const entry of rawSearch.split('&')) {
        if (!entry) {
            continue;
        }

        const separatorIndex = entry.indexOf('=');
        const rawKey = separatorIndex >= 0 ? entry.slice(0, separatorIndex) : entry;

        if (!rawKey) {
            continue;
        }

        const rawValue = separatorIndex >= 0 ? entry.slice(separatorIndex + 1) : '';
        query[safeDecodeURIComponent(rawKey)] = safeDecodeURIComponent(rawValue);
    }

    return query;
}

function parseLocation(path) {
    const normalizedPath = normalizePath(path);
    const queryIndex = normalizedPath.indexOf('?');

    if (queryIndex < 0) {
        return {
            path: normalizedPath,
            pathname: normalizedPath,
            query: {}
        };
    }

    return {
        path: normalizedPath,
        pathname: normalizedPath.slice(0, queryIndex),
        query: parseQuery(normalizedPath.slice(queryIndex + 1))
    };
}

function normalizeRoutePattern(value) {
    const rawValue = String(value ?? '').trim();
    const hashIndex = rawValue.indexOf('#');
    const withoutHash = hashIndex >= 0 ? rawValue.slice(0, hashIndex) : rawValue;
    const queryIndex = withoutHash.indexOf('?');
    const pathname = queryIndex >= 0 ? withoutHash.slice(0, queryIndex) : withoutHash;

    return normalizePathname(pathname);
}

function matchRoutePath(routePath, pathname) {
    const normalizedRoutePath = normalizeRoutePattern(routePath);
    const normalizedPathname = normalizePathname(pathname);

    if (normalizedRoutePath === '/' && normalizedPathname === '/') {
        return {};
    }

    const routeSegments = normalizedRoutePath.split('/').filter(Boolean);
    const pathSegments = normalizedPathname.split('/').filter(Boolean);

    if (routeSegments.length !== pathSegments.length) {
        return null;
    }

    const params = {};

    for (let index = 0; index < routeSegments.length; index += 1) {
        const routeSegment = routeSegments[index];
        const pathSegment = pathSegments[index];

        if (routeSegment.startsWith(':') && routeSegment.length > 1) {
            params[routeSegment.slice(1)] = safeDecodeURIComponent(pathSegment);
            continue;
        }

        if (routeSegment !== pathSegment) {
            return null;
        }
    }

    return params;
}

function createDefaultNotFoundNode(context) {
    return View({
        style: {
            width: 'fill',
            padding: 20,
            flexDirection: 'column',
            gap: 8
        },
        children: [
            Text({
                text: 'Route not found',
                style: {
                    fontSize: 24,
                    color: '#7A1F1F'
                }
            }),
            Text({
                text: `No route matched "${context.path}".`,
                style: {
                    fontSize: 16,
                    color: '#5A5F73'
                }
            })
        ]
    });
}

class AppRouter {
    constructor(app, config = {}) {
        this.app = app;
        this.routes = Array.isArray(config.routes) ? config.routes : [];
        this.notFound =
            typeof config.notFound === 'function'
                ? config.notFound
                : createDefaultNotFoundNode;

        const initialPath = normalizePath(config.initialPath ?? '/');

        this.currentPath = initialPath;
        this.historyStack = [initialPath];
        this.historyIndex = 0;
        this.navigationApi = {
            navigate: (path) => this.navigate(path),
            replace: (path) => this.replace(path),
            back: () => this.back(),
            forward: () => this.forward()
        };
    }

    getPath() {
        return this.currentPath;
    }

    navigate(path) {
        const nextPath = normalizePath(path);

        this.historyStack = this.historyStack.slice(0, this.historyIndex + 1);
        this.historyStack.push(nextPath);
        this.historyIndex = this.historyStack.length - 1;
        this.currentPath = nextPath;
        this.app.requestRender();
    }

    replace(path) {
        const nextPath = normalizePath(path);

        if (this.historyStack.length === 0) {
            this.historyStack = [nextPath];
            this.historyIndex = 0;
        } else {
            this.historyStack[this.historyIndex] = nextPath;
        }

        this.currentPath = nextPath;
        this.app.requestRender();
    }

    back() {
        if (this.historyIndex <= 0) {
            return;
        }

        this.historyIndex -= 1;
        this.currentPath = this.historyStack[this.historyIndex];
        this.app.requestRender();
    }

    forward() {
        if (this.historyIndex >= this.historyStack.length - 1) {
            return;
        }

        this.historyIndex += 1;
        this.currentPath = this.historyStack[this.historyIndex];
        this.app.requestRender();
    }

    render() {
        const location = parseLocation(this.currentPath);

        for (const route of this.routes) {
            if (!route || typeof route.render !== 'function') {
                continue;
            }

            const params = matchRoutePath(route.path, location.pathname);

            if (!params) {
                continue;
            }

            return route.render(this.createRouteContext(location, params));
        }

        return this.notFound(this.createRouteContext(location, {}));
    }

    createRouteContext(location, params) {
        return {
            path: location.path,
            params,
            query: location.query,
            navigate: this.navigationApi.navigate,
            replace: this.navigationApi.replace,
            back: this.navigationApi.back,
            forward: this.navigationApi.forward
        };
    }
}

globalThis.Navigation = {
    createRouter: (config = {}) => globalThis.App.createRouter(config),
    normalizePath,
    parseLocation,
    matchRoute: (routePath, pathname) => matchRoutePath(routePath, pathname)
};

globalThis.View = (props) => createNode('View', props);
globalThis.Text = (props) => createNode('Text', props);
globalThis.Button = (props) => createNode('Button', props);
globalThis.TextInput = (props) => createNode('TextInput', props);
globalThis.SelectInput = (props) => createNode('SelectInput', props);
globalThis.NativeSelect = (props) => createNode('SelectInput', props);
globalThis.FlatList = (props) => createFlatList(props);
globalThis.NativeList = (props) => createNativeList(props);
globalThis.NativeCombobox = (props) => createNativeCombobox(props);
globalThis.Tabs = (props) => createTabs(props);
globalThis.Modal = (props) => createNode('Modal', props);
globalThis.Alert = (props) => triggerAlert(props);
globalThis.Toast = {
    show: (props) => triggerToast(props),
    dismiss: (id) => dismissToast(id),
    clear: () => clearToasts()
};

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

globalThis.Timer = {
    after(delayMs) {
        return new Promise((resolve, reject) => {
            const requestId = PendingTimerRegistryInstance.register({
                resolve,
                reject
            });

            dispatchToRust({
                action: 'TIMER_REQUEST',
                requestId,
                delayMs: Math.max(0, Number(delayMs) || 0)
            });
        });
    }
};

function storageRequest(operation, key, value) {
    return new Promise((resolve, reject) => {
        const requestId = PendingStorageRegistryInstance.register({
            resolve,
            reject
        });

        dispatchToRust({
            action: 'STORAGE_REQUEST',
            requestId,
            operation,
            ...(key === undefined ? {} : { key: String(key) }),
            ...(value === undefined ? {} : { value: String(value) })
        });
    });
}

globalThis.Storage = {
    get(key) {
        return storageRequest('GET', key);
    },
    set(key, value) {
        return storageRequest('SET', key, value);
    },
    remove(key) {
        return storageRequest('REMOVE', key);
    },
    clear() {
        return storageRequest('CLEAR');
    }
};

class AppEngine {
    constructor() {
        this.rootRenderFn = null;
        this.isRenderPending = false;
        this.hookStates = [];
        this.hookEffects = [];
        this.hookCursor = 0;
        this.pendingEffects = [];
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
            GlobalCallbackRegistry.beginRender();
            this.hookCursor = 0;
            HookRuntime.setActiveEngine(this);

            const vdomTree = this.rootRenderFn();
            const alertNode = renderActiveAlert();
            const toastNode = renderActiveToasts();
            const tree = alertNode || toastNode
                ? View({
                    style: {
                        width: 'fill',
                        height: 'fill',
                        direction: 'column',
                        gap: 12
                    },
                    children: [vdomTree, toastNode, alertNode]
                })
                : vdomTree;

            __SEND_TO_RUST__(JSON.stringify({
                action: 'UPDATE_VDOM',
                tree
            }));

            this.flushEffects();
        } finally {
            HookRuntime.clearActiveEngine();
            GlobalCallbackRegistry.clearStale();
            this.isRenderPending = false;
        }
    }

    nextHookIndex() {
        const index = this.hookCursor;
        this.hookCursor += 1;
        return index;
    }

    useState(initialValue) {
        const hookIndex = this.nextHookIndex();

        if (!(hookIndex in this.hookStates)) {
            this.hookStates[hookIndex] =
                typeof initialValue === 'function' ? initialValue() : initialValue;
        }

        const setState = (nextValue) => {
            const previousState = this.hookStates[hookIndex];
            const resolvedValue =
                typeof nextValue === 'function' ? nextValue(previousState) : nextValue;

            if (Object.is(previousState, resolvedValue)) {
                return;
            }

            this.hookStates[hookIndex] = resolvedValue;
            this.requestRender();
        };

        return {
            state: this.hookStates[hookIndex],
            setState
        };
    }

    useEffect(effectFn, dependencies) {
        const hookIndex = this.nextHookIndex();
        const previous = this.hookEffects[hookIndex];
        const deps = Array.isArray(dependencies) ? dependencies : undefined;
        const shouldRun =
            !previous ||
            !deps ||
            !Array.isArray(previous.dependencies) ||
            deps.length !== previous.dependencies.length ||
            deps.some((dep, index) => !Object.is(dep, previous.dependencies[index]));

        this.hookEffects[hookIndex] = {
            dependencies: deps
        };

        if (shouldRun && typeof effectFn === 'function') {
            this.pendingEffects.push(effectFn);
        }
    }

    flushEffects() {
        if (this.pendingEffects.length === 0) {
            return;
        }

        const effectsToRun = this.pendingEffects;
        this.pendingEffects = [];

        for (const effectFn of effectsToRun) {
            effectFn();
        }
    }

    createRouter(config = {}) {
        return new AppRouter(this, config);
    }
}

globalThis.App = new AppEngine();

const HookRuntime = {
    activeEngine: null,

    setActiveEngine(engine) {
        this.activeEngine = engine;
    },

    clearActiveEngine() {
        this.activeEngine = null;
    },

    requireActiveEngine(name) {
        if (!this.activeEngine) {
            throw new Error(`${name}() can only be used while App is rendering.`);
        }

        return this.activeEngine;
    }
};

globalThis.useState = (initialValue) =>
    HookRuntime.requireActiveEngine('useState').useState(initialValue);

globalThis.useEffect = (effectFn, dependencies) =>
    HookRuntime.requireActiveEngine('useEffect').useEffect(effectFn, dependencies);
