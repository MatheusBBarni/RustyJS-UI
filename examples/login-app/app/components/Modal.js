import { AppButton } from './Button.js';
import { CardHeader, Surface } from './Card.js';

function toNodeArray(value) {
    if (Array.isArray(value)) {
        return value;
    }

    if (value === undefined || value === null) {
        return [];
    }

    return [value];
}

export function Dialog(props = {}) {
    return Modal({
        visible: props.visible !== false,
        onRequestClose: props.onRequestClose,
        backdropColor: props.backdropColor || '#00000066',
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            justifyContent: 'center',
            alignItems: 'center',
            ...(props.overlayStyle || {})
        },
        children: [
            Surface({
                width: props.width || 440,
                padding: props.padding || 22,
                gap: props.gap || 16,
                borderRadius: props.borderRadius || 18,
                children: [
                    ...(props.title || props.subtitle || props.actions
                        ? [
                              CardHeader({
                                  title: props.title,
                                  subtitle: props.subtitle,
                                  actions: props.actions || []
                              })
                          ]
                        : []),
                    ...toNodeArray(props.children)
                ]
            })
        ]
    });
}

export function DialogActions(props = {}) {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'row',
            justifyContent: 'end',
            alignItems: 'center',
            gap: 12,
            ...(props.style || {})
        },
        children: [
            ...(props.cancelText
                ? [
                      AppButton({
                          text: props.cancelText || 'Cancel',
                          variant: 'secondary',
                          onClick: props.onCancel,
                          disabled: Boolean(props.cancelDisabled)
                      })
                  ]
                : []),
            ...(props.confirmText
                ? [
                      AppButton({
                          text: props.confirmText || 'Save',
                          variant: props.confirmVariant || 'primary',
                          onClick: props.onConfirm,
                          disabled: Boolean(props.confirmDisabled)
                      })
                  ]
                : [])
        ]
    });
}
