let nextTaskId = 3;
let draft = '';
let tasks = [
    { id: '1', title: 'Ship FlatList', completed: false },
    { id: '2', title: 'Write integration tests', completed: true }
];

function handleDraftChange(nextValue) {
    draft = nextValue;
    App.requestRender();
}

function addTask() {
    const title = draft.trim();

    if (!title) {
        return;
    }

    tasks = [
        ...tasks,
        {
            id: String(nextTaskId++),
            title,
            completed: false
        }
    ];
    draft = '';
    App.requestRender();
}

function toggleTask(taskId) {
    tasks = tasks.map((task) => {
        if (task.id !== taskId) {
            return task;
        }

        return {
            ...task,
            completed: !task.completed
        };
    });
    App.requestRender();
}

function deleteTask(taskId) {
    tasks = tasks.filter((task) => task.id !== taskId);
    App.requestRender();
}

function taskStatus(task) {
    return task.completed ? 'Completed' : 'Pending';
}

function EmptyTasks() {
    return View({
        style: {
            width: 'fill',
            padding: 18,
            alignItems: 'center',
            borderWidth: 1,
            borderRadius: 12,
            borderColor: '#D8E0E8',
            backgroundColor: '#FFFFFF'
        },
        children: [
            Text({
                text: 'No tasks yet',
                style: {
                    fontSize: 18,
                    color: '#58677A'
                }
            })
        ]
    });
}

function renderTask({ item, index }) {
    return View({
        style: {
            width: 'fill',
            direction: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 12,
            padding: { x: 16, y: 14 },
            backgroundColor: item.completed ? '#EAF7EE' : '#FFFFFF',
            borderWidth: 1,
            borderRadius: 12,
            borderColor: item.completed ? '#ACD6B9' : '#D8E0E8'
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
                        text: `Status: ${taskStatus(item)}`,
                        style: {
                            fontSize: 16,
                            color: item.completed ? '#2C7A4B' : '#58677A'
                        }
                    })
                ]
            }),
            View({
                style: {
                    direction: 'row',
                    gap: 8
                },
                children: [
                    Button({
                        text: item.completed ? `Undo ${index + 1}` : `Complete ${index + 1}`,
                        onClick: () => toggleTask(item.id),
                        style: {
                            padding: { x: 12, y: 10 },
                            borderRadius: 10,
                            backgroundColor: item.completed ? '#6E7E8D' : '#2C7A4B',
                            color: '#FFFFFF'
                        }
                    }),
                    Button({
                        text: `Delete ${index + 1}`,
                        onClick: () => deleteTask(item.id),
                        style: {
                            padding: { x: 12, y: 10 },
                            borderRadius: 10,
                            backgroundColor: '#B13A3A',
                            color: '#FFFFFF'
                        }
                    })
                ]
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
                text: 'Task Form FlatList',
                style: {
                    fontSize: 28,
                    color: '#102033'
                }
            }),
            Text({
                text: `Tasks: ${tasks.length}`,
                style: {
                    fontSize: 18,
                    color: '#425466'
                }
            }),
            View({
                style: {
                    width: 'fill',
                    direction: 'row',
                    gap: 12,
                    alignItems: 'center'
                },
                children: [
                    TextInput({
                        value: draft,
                        placeholder: 'Task name',
                        onChange: handleDraftChange,
                        style: {
                            width: 200,
                            padding: 12,
                            borderWidth: 1,
                            borderRadius: 12,
                            borderColor: '#C7D2DE',
                            backgroundColor: '#FFFFFF',
                            color: '#102033'
                        }
                    }),
                    Button({
                        text: 'Add Task',
                        onClick: addTask,
                        style: {
                            padding: { x: 16, y: 12 },
                            borderRadius: 12,
                            backgroundColor: '#215B9A',
                            color: '#FFFFFF'
                        }
                    })
                ]
            }),
            FlatList({
                data: tasks,
                style: {
                    width: 'fill'
                },
                contentContainerStyle: {
                    gap: 12
                },
                ListEmptyComponent: EmptyTasks,
                renderItem: renderTask
            })
        ]
    });
}

App.run({
    title: 'Task Form FlatList Example',
    windowSize: { width: 760, height: 560 },
    render: AppLayout
});
