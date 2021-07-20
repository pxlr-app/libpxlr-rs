import {
	Accessor,
	batch,
	Component,
	createContext,
	createEffect,
	createSignal,
	onCleanup,
	onMount,
	useContext,
} from "solid-js";

export type Orientation = "horizontal" | "vertical";

export type NavigationDevice = "pointer" | "keyboard";

export type Setter<T> = (v: T | ((prev: T) => T)) => T;

export type AddMenuItem = {
	id: string;
	accessKey: string;
	action?: () => void;
};

export type MenuContextData = {
	orientation: Orientation;
	showAccessKey: Accessor<boolean>;
	setShowAccessKey: Setter<boolean>;
	navigationInput: Accessor<NavigationDevice>;
	setNavigationInput: Setter<NavigationDevice>;
	selected: Accessor<string | undefined>;
	select: Setter<string | undefined>;
	opened: Accessor<string | undefined>;
	open: Setter<string | undefined>;
	addMenuItem: (item: AddMenuItem) => void;
};

type AccessibilityData = Pick<
	MenuContextData,
	| "showAccessKey"
	| "setShowAccessKey"
	| "navigationInput"
	| "setNavigationInput"
>;

const MenuContext = createContext<MenuContextData | undefined>(undefined);

export type CreateMenuParams = {
	/**
	 * The HTML Element to attach keyboard events to
	 */
	accessibilityContainer?: HTMLElement;

	orientation: Orientation;
};

export function createMenu({
	orientation,
	accessibilityContainer,
}: CreateMenuParams) {
	const accessible: AccessibilityData =
		useContext(MenuContext) ??
		(() => {
			const [showAccessKey, setShowAccessKey] = createSignal(false);
			const [
				navigationInput,
				setNavigationInput,
			] = createSignal<NavigationDevice>("pointer");

			onMount(() => {
				const onKeyDown = (e: KeyboardEvent) => {
					if (e.code === "AltLeft") {
						e.preventDefault();
						e.stopImmediatePropagation();
						batch(() => {
							setShowAccessKey((state) => !state);
							setNavigationInput("keyboard");
						});
					}
				};

				const a11yContainer = accessibilityContainer ?? document.body;
				a11yContainer.addEventListener("keydown", onKeyDown);

				onCleanup(() => {
					a11yContainer.removeEventListener("keydown", onKeyDown);
				});
			});

			return {
				showAccessKey,
				setShowAccessKey,
				navigationInput,
				setNavigationInput,
			};
		})();

	const [selected, select] = createSignal<string | undefined>(undefined);
	const [opened, open] = createSignal<string | undefined>(undefined);
	const items: AddMenuItem[] = [];

	onMount(() => {
		const onLeave = (e: MouseEvent | KeyboardEvent) => {
			batch(() => {
				accessible.setNavigationInput("pointer");
				select(undefined);
				open(undefined);
			});
		};

		document.addEventListener("click", onLeave);
		document.addEventListener("keydown", onLeave);

		onCleanup(() => {
			document.removeEventListener("click", onLeave);
			document.removeEventListener("keydown", onLeave);
		});
	});

	const menu: MenuContextData = {
		...accessible,
		orientation,
		selected,
		select,
		opened,
		open,
		addMenuItem(item: AddMenuItem) {
			items.push(item);
		},
	};

	const NestedContextProvider: Component = (props) => {
		return (
			<MenuContext.Provider value={menu}>
				{props.children}
			</MenuContext.Provider>
		);
	};

	return {
		...menu,

		NestedContextProvider,

		attributes: {
			tabIndex: 0,
			onKeyDown(e: KeyboardEvent) {
				// Toggle AccessKey
				if (e.code === "AltLeft") {
					e.preventDefault();
					e.stopImmediatePropagation();
					batch(() => {
						menu.setShowAccessKey((state) => !state);
						menu.setNavigationInput("keyboard");
					});
				}
				// No item opened, process key
				else if (!menu.opened()) {
					let selectedIdx = items.findIndex(
						(item) => menu.selected() === item.id,
					);
					// Down
					if (
						(menu.orientation === "vertical" &&
							e.code === "ArrowDown") ||
						(menu.orientation === "horizontal" &&
							e.code === "ArrowRight")
					) {
						e.preventDefault();
						e.stopImmediatePropagation();
						selectedIdx = (selectedIdx + 1) % items.length;
						batch(() => {
							menu.setNavigationInput("keyboard");
							menu.select(items[selectedIdx].id);
						});
					}
					// Up
					else if (
						(menu.orientation === "vertical" &&
							e.code === "ArrowUp") ||
						(menu.orientation === "horizontal" &&
							e.code === "ArrowLeft")
					) {
						e.preventDefault();
						e.stopImmediatePropagation();
						if (selectedIdx === -1) {
							selectedIdx = items.length - 1;
						} else {
							selectedIdx =
								(items.length + (selectedIdx - 1)) %
								items.length;
						}
						batch(() => {
							menu.setNavigationInput("keyboard");
							menu.select(items[selectedIdx].id);
						});
					}
					// Back
					else if (
						(menu.orientation === "vertical" &&
							e.code === "ArrowLeft") ||
						(menu.orientation === "horizontal" &&
							e.code === "ArrowUp")
					) {
						batch(() => {
							menu.setNavigationInput("keyboard");
							menu.select(undefined);
							menu.open(undefined);
						});
					}
					// AccessKey?
					else if (menu.showAccessKey()) {
						const accessedItem = items.find(
							({ accessKey }) =>
								`Key${accessKey.toUpperCase()}` === e.code,
						);
						if (accessedItem) {
							if (accessedItem.action) {
								batch(() => {
									menu.setShowAccessKey(false);
									menu.setNavigationInput("keyboard");
									menu.select(undefined);
									menu.open(undefined);
								});
								accessedItem.action();
							} else {
								e.preventDefault();
								e.stopImmediatePropagation();
								batch(() => {
									menu.setNavigationInput("keyboard");
									menu.select(accessedItem.id);
									menu.open(accessedItem.id);
								});
							}
						}
					}
				}
				// Back
				else if (
					(menu.orientation === "vertical" &&
						e.code === "ArrowLeft") ||
					(menu.orientation === "horizontal" && e.code === "ArrowUp")
				) {
					e.preventDefault();
					e.stopImmediatePropagation();
					batch(() => {
						menu.setNavigationInput("keyboard");
						menu.open(undefined);
					});
				}
			},
		},
	};
}

export type CreateMenuItemParams = {
	/**
	 * A unique identifier for this menu item
	 */
	id: string;
	/**
	 * The access key used for accessibility navigation
	 */
	accessKey: string;
	/**
	 * The action to execute when clicking on the menu item
	 */
	action?: () => void;
};

export function createMenuItem({
	id,
	accessKey,
	action,
}: CreateMenuItemParams) {
	const menu = useContext(MenuContext);
	if (!menu) {
		throw new Error(
			"Could not find MenuContext. Did you forget to call createMenu this component's line?",
		);
	}

	menu.addMenuItem({ id, accessKey, action });

	const onClick = (e: MouseEvent | KeyboardEvent) => {
		// Not bubbling up
		if (e.target === e.currentTarget) {
			// Has action
			if (action) {
				batch(() => {
					menu.select(undefined);
					menu.open(undefined);
				});
				action();
			}
			// Has children
			else {
				e.preventDefault();
				e.stopImmediatePropagation();
				if (menu.opened() === id) {
					batch(() => {
						// menu.select(undefined);
						menu.open(undefined);
					});
				} else {
					batch(() => {
						menu.select(id);
						menu.open(id);
					});
				}
			}
		}
	};

	return {
		...menu,

		attributes: {
			tabIndex: -1,
			ref(elem: HTMLElement) {
				createEffect(() => {
					if (menu.selected() === id) {
						elem.focus();
					} else if (elem === document.activeElement) {
						elem.blur();
					}
				});
			},
			onPointerEnter(e: PointerEvent) {
				batch(() => {
					menu.setNavigationInput("pointer");
					menu.select(id);
					if (menu.selected() !== id) {
						menu.open(undefined);
					}
				});
			},
			onFocus(e: FocusEvent) {
				batch(() => {
					menu.select(id);
					if (menu.selected() !== id) {
						menu.open(undefined);
					}
				});
			},
			onClick(e: MouseEvent) {
				menu.setNavigationInput("pointer");
				onClick(e);
			},
			onKeyDown(e: KeyboardEvent) {
				if (e.code === "AltLeft") {
					e.preventDefault();
					e.stopImmediatePropagation();
					batch(() => {
						menu.setShowAccessKey((state) => !state);
						menu.setNavigationInput("keyboard");
					});
				} else if (
					e.code === "Space" ||
					e.code === "Enter" ||
					(menu.orientation === "vertical" &&
						e.code === "ArrowRight") ||
					(menu.orientation === "horizontal" &&
						e.code === "ArrowDown")
				) {
					menu.setNavigationInput("keyboard");
					onClick(e);
				}
			},
		},
	};
}