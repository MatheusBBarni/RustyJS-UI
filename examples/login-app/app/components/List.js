function toNodeArray(value) {
    if (Array.isArray(value)) {
        return value;
    }

    if (value === undefined || value === null) {
        return [];
    }

    return [value];
}

function appendText(children, text, style) {
    if (!text) {
        return;
    }

    children.push(
        Text({
            text,
            style
        })
    );
}

function defaultEmptyComponent(props) {
    return EmptyState({
        title: props.emptyTitle || 'Nothing here yet',
        description: props.emptyDescription || 'There are no items to display.'
    });
}

export function EmptyState(props = {}) {
    const children = [];

    appendText(children, props.title, {
        fontSize: 18,
        color: props.titleColor || '#17324D'
    });

    appendText(children, props.description, {
        fontSize: 15,
        color: props.descriptionColor || '#5A6C7F'
    });

    return View({
        style: {
            width: 'fill',
            padding: props.padding ?? 18,
            flexDirection: 'column',
            gap: 8,
            alignItems: 'center',
            justifyContent: 'center',
            borderWidth: 1,
            borderRadius: 14,
            borderColor: props.borderColor || '#D8E0E8',
            backgroundColor: props.backgroundColor || '#FFFFFF',
            ...(props.style || {})
        },
        children
    });
}

export function ListSection(props = {}) {
    const children = [];

    if (props.header) {
        children.push(
            View({
                style: {
                    width: 'fill'
                },
                children: toNodeArray(props.header)
            })
        );
    }

    children.push(
        FlatList({
            data: Array.isArray(props.data) ? props.data : [],
            keyExtractor: props.keyExtractor,
            horizontal: Boolean(props.horizontal),
            style: {
                width: 'fill',
                height: props.listHeight || 'fill',
                ...(props.listStyle || {})
            },
            contentContainerStyle: {
                gap: props.itemGap ?? 12,
                ...(props.contentContainerStyle || {})
            },
            ListHeaderComponent: props.ListHeaderComponent,
            ListFooterComponent: props.ListFooterComponent,
            ListEmptyComponent: props.ListEmptyComponent || (() => defaultEmptyComponent(props)),
            ItemSeparatorComponent: props.ItemSeparatorComponent,
            renderItem: props.renderItem
        })
    );

    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 12,
            ...(props.style || {})
        },
        children
    });
}

export function EntityRow(props = {}) {
    const actions = [];

    if (Array.isArray(props.actions)) {
        actions.push(...props.actions);
    } else if (props.actions) {
        actions.push(props.actions);
    }

    const detailsChildren = [];
    appendText(detailsChildren, props.title, {
        fontSize: 18,
        color: '#102033'
    });
    appendText(detailsChildren, props.subtitle, {
        fontSize: 15,
        color: '#425466'
    });
    appendText(detailsChildren, props.description, {
        fontSize: 14,
        color: '#6B7D90'
    });

    const rowChildren = [
        View({
            style: {
                flexDirection: 'column',
                gap: 4
            },
            children: detailsChildren
        })
    ];

    if (props.badge || actions.length > 0) {
        const metaChildren = [];

        if (props.badge) {
            metaChildren.push(props.badge);
        }

        if (actions.length > 0) {
            metaChildren.push(
                View({
                    style: {
                        flexDirection: 'row',
                        gap: 8
                    },
                    children: actions
                })
            );
        }

        rowChildren.push(
            View({
                style: {
                    flexDirection: 'column',
                    alignItems: 'end',
                    gap: 8
                },
                children: metaChildren
            })
        );
    }

    return View({
        style: {
            width: 'fill',
            flexDirection: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 12,
            padding: 16,
            borderWidth: 1,
            borderRadius: 14,
            borderColor: '#D7E0EA',
            backgroundColor: '#FFFFFF',
            ...(props.style || {})
        },
        children: rowChildren
    });
}
