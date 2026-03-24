const tasks = [
    { id: 'ship-flat-list', title: 'Ship FlatList', owner: 'Ada' },
    { id: 'review-bridge', title: 'Review JS bridge', owner: 'Grace' },
    { id: 'polish-renderer', title: 'Polish renderer', owner: 'Linus' },
];

let selectedTaskId = '';

function getSelectedTask() {
    return tasks.find((task) => task.id === selectedTaskId) || null;
}

function getSelectedTaskLabel() {
    const task = getSelectedTask();
    return task ? task.title : 'Nothing selected';
}

function selectTask(taskId) {
    selectedTaskId = taskId;
    App.requestRender();
}

function Divider() {
    return View({
        style: {
            width: 'fill',
            height: 1,
            backgroundColor: '#D8E0E8'
        }
    });
}

function renderTask({ item, index }) {
    return View({
        style: {
            width: 'fill',
            direction: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            padding: { x: 14, y: 12 },
            backgroundColor: '#FFFFFF',
            borderWidth: 1,
            borderRadius: 12,
            borderColor: '#D8E0E8'
        },
        children: [
            View({
                style: {
                    direction: 'column',
                    gap: 4
                },
                children: [
                    Text({
                        text: `${index + 1}. ${item.title}`,
                        style: {
                            fontSize: 20,
                            color: '#102033'
                        }
                    }),
                    Text({
                        text: `Owner: ${item.owner}`,
                        style: {
                            fontSize: 16,
                            color: '#58677A'
                        }
                    })
                ]
            }),
            Button({
                text: `Select ${index + 1}`,
                onClick: () => selectTask(item.id),
                style: {
                    padding: { x: 12, y: 10 },
                    borderRadius: 10,
                    backgroundColor: '#215B9A',
                    color: '#FFFFFF'
                }
            })
        ]
    });
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            direction: 'column',
            gap: 16,
            backgroundColor: '#F4F7FA'
        },
        children: [
            Text({
                text: 'FlatList Example',
                style: {
                    fontSize: 28,
                    color: '#102033'
                }
            }),
            Text({
                text: `Selected task: ${getSelectedTaskLabel()}`,
                style: {
                    fontSize: 18,
                    color: '#425466'
                }
            }),
            FlatList({
                data: tasks,
                style: {
                    width: 'fill'
                },
                contentContainerStyle: {
                    gap: 12
                },
                ItemSeparatorComponent: Divider,
                renderItem: renderTask
            })
        ]
    });
}

App.run({
    title: 'FlatList Example',
    windowSize: { width: 720, height: 520 },
    render: AppLayout
});
