import React from "react";
import { Story, Meta } from "@storybook/react";
import { Menu, MenuItem, Separator, ControlledMenu } from "./Menu";

export default {
	title: "Layout/Menu",
	component: Menu,
	argTypes: {
		//   backgroundColor: { control: 'color' },
	},
} as Meta;

const Template = () => (
	<Menu width="300px">
		<MenuItem
			id="newfile"
			label="New File"
			accesskey="N"
			keybind="Ctrl+N"
			action={() => console.log("newfile")}
		/>
		<MenuItem
			id="newwindow"
			label="New Window"
			accesskey="W"
			keybind="Ctrl+Shift+N"
			action={() => console.log("newwindow")}
		/>
		<Separator />
		<MenuItem
			id="openfile"
			label="Open File…"
			accesskey="O"
			keybind="Ctrl+O"
			action={() => console.log("openfile")}
		/>
		<MenuItem
			id="openrecent"
			label="Open Recent"
			accesskey="R"
			keybind="Ctrl+Shift+O"
			action={() => console.log("openrecent")}
		/>
		<Separator />
		<MenuItem
			id="save"
			label="Save"
			accesskey="S"
			keybind="Ctrl+S"
			action={() => console.log("save")}
		/>
		<MenuItem
			id="saveas"
			label="Save As…"
			accesskey="A"
			keybind="Ctrl+Shift+S"
			action={() => console.log("saveas")}
		/>
		<MenuItem
			id="autosave"
			label="Auto Save"
			accesskey="t"
			checked
			action={() => console.log("autosave")}
		/>
		<Separator />
		<MenuItem id="preferences" label="Preferences" accesskey="P">
			<Menu width="300px">
				<MenuItem
					id="settings"
					label="Settings"
					accesskey="S"
					keybind="Ctrl+,"
					action={() => console.log("settings")}
				/>
				<MenuItem
					id="keyboardshortcuts"
					label="Keyboard Shortcuts"
					accesskey="K"
					action={() => console.log("keyboardshortcuts")}
				/>
			</Menu>
		</MenuItem>
		<MenuItem
			id="useraccount"
			label="User Account"
			accesskey="U"
			action={() => console.log("useraccount")}
		/>
	</Menu>
);

export const Uncontrolled: Story<{}> = (args) => <Template />;

export const Controlled: Story<{}> = (args) => (
	<ControlledMenu containerRef={document as any}>
		<Template />
	</ControlledMenu>
);