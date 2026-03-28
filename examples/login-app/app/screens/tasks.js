import {
    AppButton,
    Badge,
    CardHeader,
    EntityRow,
    ListSection,
    ScreenShell,
    SelectField,
    Surface,
    TextField
} from '../components/index.js';
import { countLabel, formatDateTime, summarizeText, taskStatusLabel } from '../lib/format.js';
import { matchesTaskFilter, TASK_FILTER_OPTIONS } from '../lib/options.js';
import {
    createTaskFromComposer,
    ensureTasksLoaded,
    isAuthenticated,
    refreshTasks,
    requestTaskRemoval,
    setTaskComposerField,
    setTaskFilter,
    state
} from '../state/app-state.js';

function TaskRow(route, task) {
    return EntityRow({
        title: task.title,
        subtitle: `${taskStatusLabel(task.completed)} | Updated ${formatDateTime(task.updatedAt)}`,
        description: summarizeText(task.description),
        badge: Badge({
            variant: task.completed ? 'success' : 'warning',
            text: task.completed ? 'Completed' : 'Open'
        }),
        style: {
            backgroundColor: task.completed ? '#F1FAF4' : '#FFFFFF',
            borderColor: task.completed ? '#B7DCC6' : '#DCCDBE'
        },
        actions: [
            AppButton({
                text: 'Show',
                size: 'sm',
                variant: 'ghost',
                onClick: () => route.navigate(`/tasks/${task.id}?mode=view`)
            }),
            AppButton({
                text: 'Edit',
                size: 'sm',
                variant: 'secondary',
                onClick: () => route.navigate(`/tasks/${task.id}?mode=edit`)
            }),
            AppButton({
                text: state.loading.deletingTaskId === task.id ? 'Deleting...' : 'Delete',
                size: 'sm',
                variant: 'danger',
                disabled: state.loading.deletingTaskId === task.id,
                onClick: () => requestTaskRemoval(task.id)
            })
        ]
    });
}

function ListHeader(filteredTasks) {
    const total = state.tasks.length;
    const completed = state.tasks.filter((task) => task.completed).length;
    const open = total - completed;

    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 12
        },
        children: [
            Surface({
                backgroundColor: '#FFF8EC',
                borderColor: '#E9D2AA',
                children: [
                    CardHeader({
                        title: 'Create a task',
                        subtitle: 'Add tasks directly from the protected list route.'
                    }),
                    TextField({
                        label: 'Title',
                        value: state.forms.createTask.title,
                        placeholder: 'Ship the login app example',
                        onChange: (value) => setTaskComposerField('title', value)
                    }),
                    TextField({
                        label: 'Description',
                        value: state.forms.createTask.description,
                        placeholder: 'Optional description',
                        multiline: true,
                        onChange: (value) => setTaskComposerField('description', value),
                        inputStyle: {
                            height: 110
                        }
                    }),
                    View({
                        style: {
                            width: 'fill',
                            flexDirection: 'row',
                            gap: 12
                        },
                        children: [
                            AppButton({
                                text: state.loading.taskCreate ? 'Creating...' : 'Create task',
                                disabled: state.loading.taskCreate,
                                onClick: createTaskFromComposer
                            }),
                            AppButton({
                                text: 'Refresh',
                                variant: 'secondary',
                                onClick: refreshTasks
                            })
                        ]
                    })
                ]
            }),
            Surface({
                backgroundColor: '#FFFFFF',
                borderColor: '#DCCDBE',
                children: [
                    CardHeader({
                        title: 'Task filters',
                        subtitle: `${countLabel(open, 'open task')} and ${countLabel(completed, 'completed task')} are currently loaded.`,
                        actions: [
                            Badge({
                                variant: 'neutral',
                                text: `${countLabel(filteredTasks.length, 'visible task')}`
                            })
                        ]
                    }),
                    SelectField({
                        label: 'Filter',
                        value: state.filters.tasks,
                        options: TASK_FILTER_OPTIONS,
                        onChange: setTaskFilter
                    })
                ]
            })
        ]
    });
}

export function TasksScreen(route) {
    if (!isAuthenticated()) {
        route.replace('/');
        return ScreenShell({
            title: 'Redirecting',
            subtitle: 'The tasks route is protected.',
            backgroundColor: '#F7F2E8',
            children: []
        });
    }

    ensureTasksLoaded();

    const filteredTasks = state.tasks.filter((task) => matchesTaskFilter(state.filters.tasks, task));

    return ScreenShell({
        title: 'Tasks',
        subtitle: 'FlatList-backed tasks with show, edit, and delete actions.',
        backgroundColor: '#F7F2E8',
        children: [
            ListSection({
                data: filteredTasks,
                listStyle: {
                    height: 'fill'
                },
                contentContainerStyle: {
                    gap: 12
                },
                ListHeaderComponent: () => ListHeader(filteredTasks),
                emptyTitle: state.loading.tasks
                    ? 'Loading tasks'
                    : state.tasks.length === 0
                      ? 'No tasks yet'
                      : 'No tasks match this filter',
                emptyDescription: state.loading.tasks
                    ? 'Fetching tasks from the API.'
                    : state.tasks.length === 0
                      ? 'Create the first task from the composer above.'
                      : 'Try another filter or create a new task.',
                renderItem: ({ item }) => TaskRow(route, item)
            })
        ]
    });
}
