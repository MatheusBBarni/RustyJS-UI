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
            alignItems: 'center'
        },
        children: [
            Text({
                text: 'Hello world',
                style: {
                    fontSize: 28,
                    color: '#111111'
                }
            }),
            Text({
                text: `Count is: ${counter}`,
                style: {
                    fontSize: 18,
                    color: '#333333'
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
    title: 'Hello World Example',
    windowSize: { width: 480, height: 320 },
    render: AppLayout
});
