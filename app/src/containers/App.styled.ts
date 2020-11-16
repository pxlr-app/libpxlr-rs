import { createGlobalStyle } from 'styled-components';

export const Global = createGlobalStyle`
	html {
		width: 100%;
		height: 100%;
	}
	body, body > #root {
		width: 100%;
		height: 100%;
		margin: 0;
		padding: 0;
		overflow: hidden;
		font-family: Segoe WPC, Segoe UI, sans-serif;
		font-size: 11px;
		user-select: none;
		display: flex;
	}
`;