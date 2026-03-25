const VARIANTS = {
    neutral: {
        backgroundColor: '#E6EDF6',
        color: '#17324D',
        borderColor: '#C8D5E4'
    },
    success: {
        backgroundColor: '#E8F6ED',
        color: '#2C7A4B',
        borderColor: '#B7DCC6'
    },
    warning: {
        backgroundColor: '#FFF4DE',
        color: '#A86B00',
        borderColor: '#F1D29E'
    },
    danger: {
        backgroundColor: '#FBE9E9',
        color: '#9F2F2F',
        borderColor: '#E2A5A5'
    }
};

export function Badge(props = {}) {
    const variant = VARIANTS[props.variant] || VARIANTS.neutral;

    return View({
        style: {
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'center',
            padding: { x: 10, y: 6 },
            borderWidth: 1,
            borderRadius: 999,
            ...variant,
            ...(props.style || {})
        },
        children: [
            Text({
                text: props.text || '',
                style: {
                    fontSize: 13,
                    color: props.color || variant.color
                }
            })
        ]
    });
}

