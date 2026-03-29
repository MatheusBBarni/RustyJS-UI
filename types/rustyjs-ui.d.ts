declare module 'RustyJS-UI' {
  export type SizeValue = number | 'auto' | 'fill' | 'shrink';

  export interface EdgeInsetsObject {
    top?: number;
    right?: number;
    bottom?: number;
    left?: number;
    x?: number;
    y?: number;
    all?: number;
  }

  export interface Style {
    direction?: 'row' | 'column';
    flexDirection?: 'row' | 'column';
    gap?: number;
    spacing?: number;
    width?: SizeValue;
    height?: SizeValue;
    alignItems?: 'start' | 'center' | 'end' | 'stretch' | 'flex-start' | 'flex-end';
    justifyContent?:
      | 'start'
      | 'center'
      | 'end'
      | 'flex-start'
      | 'flex-end'
      | 'space-between'
      | 'space-around'
      | 'space-evenly';
    padding?: number | EdgeInsetsObject;
    backgroundColor?: string;
    borderColor?: string;
    borderWidth?: number;
    borderRadius?: number;
    color?: string;
    fontSize?: number;
    fontWeight?: 'thin' | 'light' | 'normal' | 'medium' | 'semibold' | 'bold' | 'heavy';
  }

  export type Node = unknown;

  export interface WindowSize {
    width?: number;
    height?: number;
  }

  export interface AppConfig {
    title?: string;
    windowSize?: WindowSize;
    render: () => Node;
  }

  export interface RouteContext {
    path: string;
    params: Record<string, string>;
    query: Record<string, string>;
    navigate(path: string): void;
    replace(path: string): void;
    back(): void;
    forward(): void;
  }

  export interface RouteDefinition {
    path: string;
    render(context: RouteContext): Node;
  }

  export interface RouterConfig {
    initialPath?: string;
    routes?: RouteDefinition[];
    notFound?(context: RouteContext): Node;
  }

  export interface Router {
    getPath(): string;
    navigate(path: string): void;
    replace(path: string): void;
    back(): void;
    forward(): void;
    render(): Node;
  }

  export interface AppEngine {
    run(config: AppConfig): void;
    requestRender(): void;
    createRouter(config?: RouterConfig): Router;
  }

  export const App: AppEngine;

  export const Navigation: {
    createRouter(config?: RouterConfig): Router;
    normalizePath(path: string): string;
    parseLocation(path: string): {
      path: string;
      pathname: string;
      query: Record<string, string>;
    };
    matchRoute(routePath: string, pathname: string): Record<string, string> | null;
  };

  export interface ViewProps {
    style?: Style;
    children?: Node | Node[];
  }

  export interface TextProps {
    text?: string;
    value?: string;
    style?: Style;
  }

  export interface ButtonProps {
    text?: string;
    value?: string;
    disabled?: boolean;
    onClick?: () => void;
    onPress?: () => void;
    style?: Style;
  }

  export interface TextInputProps {
    value: string;
    placeholder?: string;
    multiline?: boolean;
    type?: 'text' | 'password';
    disabled?: boolean;
    onChange?: (nextValue: string) => void;
    onValueChange?: (nextValue: string) => void;
    style?: Style;
  }

  export interface SelectOption {
    label?: string;
    value?: string;
  }

  export interface SelectInputProps {
    value: string;
    placeholder?: string;
    options?: Array<string | SelectOption>;
    disabled?: boolean;
    onChange?: (nextValue: string) => void;
    onValueChange?: (nextValue: string) => void;
    style?: Style;
  }

  export interface FlatListProps<T = unknown> {
    data?: T[];
    renderItem?: (context: { item: T; index: number }) => Node;
    horizontal?: boolean;
    keyExtractor?: (item: T, index: number) => string;
    ListHeaderComponent?: Node | ((context: { data: T[] }) => Node);
    ListFooterComponent?: Node | ((context: { data: T[] }) => Node);
    ListEmptyComponent?: Node | ((context: { data: T[] }) => Node);
    ItemSeparatorComponent?:
      | Node
      | ((context: { leadingItem: T; leadingIndex: number }) => Node);
    style?: Style;
    contentContainerStyle?: Style;
  }

  export interface NativeListProps<T = unknown> extends FlatListProps<T> {
    estimatedItemSize?: number;
    overscan?: number;
    onVisibleRangeChange?: (range: { start: number; end: number }) => void;
  }

  export interface NativeComboboxProps {
    value?: string;
    placeholder?: string;
    options?: Array<string | SelectOption>;
    allowCustomValue?: boolean;
    onChange?: (nextValue: string) => void;
    onInputChange?: (nextValue: string) => void;
    style?: Style;
    inputStyle?: Style;
    listStyle?: Style;
  }

  export interface TabsTab {
    label: string;
    value: string;
    disabled?: boolean;
    content?: Node;
    render?: (tab: TabsTab) => Node;
  }

  export interface TabsProps {
    value?: string;
    tabs?: TabsTab[];
    onChange?: (nextValue: string) => void;
    style?: Style;
    tabListStyle?: Style;
    tabStyle?: Style;
    activeTabStyle?: Style;
    panelStyle?: Style;
  }

  export interface ModalProps {
    visible?: boolean;
    transparent?: boolean;
    closeOnEscape?: boolean;
    closeOnBackdrop?: boolean;
    onRequestClose?: () => void;
    onClose?: () => void;
    backdropColor?: string;
    style?: Style;
    children?: Node | Node[];
  }

  export interface AlertProps {
    title?: string;
    description?: string;
    primaryButtonText?: string;
    primaryButtonOnClick?: () => void;
    secondaryButtonText?: string;
    secondaryButtonOnClick?: () => void;
    onClose?: () => void;
    style?: Style;
    titleStyle?: Style;
    descriptionStyle?: Style;
    buttonContainerStyle?: Style;
    primaryButtonStyle?: Style;
    secondaryButtonStyle?: Style;
  }

  export interface ToastOptions {
    message: string;
    title?: string;
    tone?: 'info' | 'success' | 'error';
    durationMs?: number;
    actionLabel?: string;
    onAction?: () => void;
    onDismiss?: () => void;
  }

  export interface FetchOptions {
    method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
    headers?: Record<string, string>;
    body?: string | Record<string, unknown>;
  }

  export const View: (props?: ViewProps) => Node;
  export const Text: (props?: TextProps) => Node;
  export const Button: (props?: ButtonProps) => Node;
  export const TextInput: (props: TextInputProps) => Node;
  export const SelectInput: (props: SelectInputProps) => Node;
  export const NativeSelect: (props: SelectInputProps) => Node;
  export const FlatList: <T = unknown>(props: FlatListProps<T>) => Node;
  export const NativeList: <T = unknown>(props: NativeListProps<T>) => Node;
  export const NativeCombobox: (props: NativeComboboxProps) => Node;
  export const Tabs: (props: TabsProps) => Node;
  export const Modal: (props: ModalProps) => Node;
  export const Alert: (props?: AlertProps) => void;
  export const Toast: {
    show(props: ToastOptions): string;
    dismiss(id: string): void;
    clear(): void;
  };
  export const Storage: {
    get(key: string): Promise<string | null>;
    set(key: string, value: string): Promise<string | null>;
    remove(key: string): Promise<string | null>;
    clear(): Promise<string | null>;
  };
  export const Timer: {
    after(delayMs: number): Promise<void>;
  };
  export const fetch: (url: string, options?: FetchOptions) => Promise<string>;

  export function useState<T>(
    initialValue: T | (() => T)
  ): { state: T; setState(nextValue: T | ((previousValue: T) => T)): void };

  export function useEffect(
    effect: () => void,
    dependencies?: readonly unknown[]
  ): void;
}
