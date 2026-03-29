function showToast() {
    Toast.show({
        message: 'Profile saved',
        tone: 'success',
        actionLabel: 'Undo',
        durationMs: 2000,
        onAction: () => {}
    });
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            direction: 'column',
            justifyContent: 'center',
            alignItems: 'center',
            gap: 18
        },
        children: [
            Text({
                text: 'Toast Example',
                style: {
                    fontSize: 28
                }
            }),
            Button({
                text: 'Show toast',
                onClick: showToast,
                style: {
                    padding: { x: 16, y: 12 },
                    borderRadius: 12,
                    backgroundColor: '#215B9A',
                    color: '#FFFFFF'
                }
            })
        ]
    });
}

App.run({
    title: 'Toast Example',
    windowSize: { width: 620, height: 360 },
    render: AppLayout
});
