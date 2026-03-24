let isModalVisible = false;

function openModal() {
    isModalVisible = true;
    App.requestRender();
}

function closeModal() {
    isModalVisible = false;
    App.requestRender();
}

function ModalActions() {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'row',
            justifyContent: 'end',
            gap: 12
        },
        children: [
            Button({
                text: 'Close',
                onClick: closeModal,
                style: {
                    padding: { x: 14, y: 12 },
                    borderRadius: 12,
                    backgroundColor: '#DDE6EF',
                    color: '#102033'
                }
            }),
            Button({
                text: 'Save',
                onClick: closeModal,
                style: {
                    padding: { x: 14, y: 12 },
                    borderRadius: 12,
                    backgroundColor: '#1E7A5F',
                    color: '#FFFFFF'
                }
            })
        ]
    });
}

function ModalCard() {
    return View({
        style: {
            width: 420,
            padding: { x: 24, y: 20 },
            flexDirection: 'column',
            gap: 14,
            backgroundColor: '#FFFFFF',
            borderWidth: 1,
            borderRadius: 18,
            borderColor: '#D5DEE8'
        },
        children: [
            Text({
                text: 'Confirm Changes',
                style: {
                    fontSize: 26,
                    color: '#102033'
                }
            }),
            Text({
                text: 'Press Escape or use one of the buttons below to dismiss this modal.',
                style: {
                    color: '#425466'
                }
            }),
            ModalActions()
        ]
    });
}

function ModalLayer() {
    return Modal({
        visible: isModalVisible,
        onRequestClose: closeModal,
        backdropColor: '#00000059',
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            justifyContent: 'center',
            alignItems: 'center'
        },
        children: [ModalCard()]
    });
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 32,
            flexDirection: 'column',
            justifyContent: 'center',
            alignItems: 'center',
            gap: 18,
            backgroundColor: '#EEF3F7'
        },
        children: [
            Text({
                text: 'Modal Example',
                style: {
                    fontSize: 30,
                    color: '#102033'
                }
            }),
            Text({
                text: 'Open the modal to see the overlay host in action.',
                style: {
                    color: '#425466'
                }
            }),
            Button({
                text: 'Open modal',
                onClick: openModal,
                style: {
                    padding: { x: 16, y: 12 },
                    borderRadius: 12,
                    backgroundColor: '#215B9A',
                    color: '#FFFFFF'
                }
            }),
            ModalLayer()
        ]
    });
}

App.run({
    title: 'Modal Example',
    windowSize: { width: 760, height: 560 },
    render: AppLayout
});
