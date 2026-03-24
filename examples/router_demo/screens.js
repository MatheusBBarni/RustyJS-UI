export function PageShell(props = {}) {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 16,
            padding: 20,
            backgroundColor: '#F4F7FA',
            borderWidth: 1,
            borderRadius: 16,
            borderColor: '#D7E0EA'
        },
        children: [
            Text({
                text: props.title || 'Untitled page',
                style: {
                    fontSize: 28,
                    color: '#102033'
                }
            }),
            ...(props.children || [])
        ]
    });
}

export function NavButton(props = {}) {
    return Button({
        text: props.text || 'Navigate',
        onClick: props.onClick,
        style: {
            padding: { x: 14, y: 12 },
            borderRadius: 12,
            backgroundColor: props.backgroundColor || '#215B9A',
            color: props.color || '#FFFFFF'
        }
    });
}
