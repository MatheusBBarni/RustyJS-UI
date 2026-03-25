function toNodeArray(value) {
    if (Array.isArray(value)) {
        return value;
    }

    if (value === undefined || value === null) {
        return [];
    }

    return [value];
}

export function Surface(props = {}) {
    return View({
        style: {
            width: props.width || 'fill',
            flexDirection: 'column',
            gap: props.gap ?? 14,
            padding: props.padding ?? 20,
            backgroundColor: props.backgroundColor || '#FFFFFF',
            borderWidth: props.borderWidth ?? 1,
            borderRadius: props.borderRadius ?? 18,
            borderColor: props.borderColor || '#D7E0EA',
            ...(props.style || {})
        },
        children: props.children || []
    });
}

export function ScreenShell(props = {}) {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: props.padding ?? 24,
            flexDirection: 'column',
            gap: props.gap ?? 18,
            backgroundColor: props.backgroundColor || '#F4F7FA',
            ...(props.style || {})
        },
        children: [
            View({
                style: {
                    flexDirection: 'column',
                    gap: 6
                },
                children: [
                    Text({
                        text: props.title || 'Untitled screen',
                        style: {
                            fontSize: props.titleSize || 32,
                            color: props.titleColor || '#102033'
                        }
                    }),
                    ...(props.subtitle
                        ? [
                              Text({
                                  text: props.subtitle,
                                  style: {
                                      fontSize: 16,
                                      color: props.subtitleColor || '#5A6C7F'
                                  }
                              })
                          ]
                        : [])
                ]
            }),
            ...toNodeArray(props.children)
        ]
    });
}

export function CardHeader(props = {}) {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 12,
            ...(props.style || {})
        },
        children: [
            View({
                style: {
                    flexDirection: 'column',
                    gap: 4
                },
                children: [
                    ...(props.title
                        ? [
                              Text({
                                  text: props.title,
                                  style: {
                                      fontSize: props.titleSize || 22,
                                      color: props.titleColor || '#102033'
                                  }
                              })
                          ]
                        : []),
                    ...(props.subtitle
                        ? [
                              Text({
                                  text: props.subtitle,
                                  style: {
                                      fontSize: 15,
                                      color: props.subtitleColor || '#5A6C7F'
                                  }
                              })
                          ]
                        : [])
                ]
            }),
            ...(Array.isArray(props.actions)
                ? props.actions
                : props.actions
                  ? [props.actions]
                  : [])
        ]
    });
}
