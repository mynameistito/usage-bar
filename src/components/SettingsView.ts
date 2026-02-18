import packageJson from "../../package.json";

const APP_VERSION = packageJson.version ?? "dev";

export interface SettingsCallbacks {
	checkZaiApiKey: () => Promise<boolean>;
	validateZaiApiKey: (apiKey: string) => Promise<void>;
	saveZaiApiKey: (apiKey: string) => Promise<void>;
	deleteZaiApiKey: () => Promise<void>;
	onZaiKeyChanged: () => Promise<void>;
	checkAmpSessionCookie: () => Promise<boolean>;
	validateAmpSessionCookie: (cookie: string) => Promise<void>;
	saveAmpSessionCookie: (cookie: string) => Promise<void>;
	deleteAmpSessionCookie: () => Promise<void>;
	onAmpCookieChanged: () => Promise<void>;
	openUrl: (url: string) => Promise<void>;
	onClose: () => void;
}

function isEnvVarSyntax(value: string): boolean {
	const lower = value.toLowerCase();
	return lower.startsWith("{env:") || lower.startsWith("$env:");
}

function createEyeIcon(crossed: boolean): SVGElement {
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.setAttribute("width", "16");
	svg.setAttribute("height", "16");
	svg.setAttribute("viewBox", "0 0 24 24");
	svg.setAttribute("fill", "none");
	svg.setAttribute("stroke", "currentColor");
	svg.setAttribute("stroke-width", "2");
	svg.setAttribute("stroke-linecap", "round");
	svg.setAttribute("stroke-linejoin", "round");

	if (crossed) {
		const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
		path.setAttribute("d", "M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24");
		svg.appendChild(path);

		const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
		line.setAttribute("x1", "1");
		line.setAttribute("y1", "1");
		line.setAttribute("x2", "23");
		line.setAttribute("y2", "23");
		svg.appendChild(line);
	} else {
		const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
		path.setAttribute("d", "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z");
		svg.appendChild(path);

		const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
		circle.setAttribute("cx", "12");
		circle.setAttribute("cy", "12");
		circle.setAttribute("r", "3");
		svg.appendChild(circle);
	}

	return svg;
}

export function createSettingsView(callbacks: SettingsCallbacks, hasZaiApiKey: boolean, hasAmpCookie: boolean): HTMLElement {
	const root = document.createElement("div");
	root.id = "settings-view";
	root.className = "settings-view";

	root.appendChild(createHeader(callbacks));

	const content = document.createElement("div");
	content.className = "settings-content";

	const zaiSection = createZaiSection(callbacks, hasZaiApiKey);
	content.appendChild(zaiSection);

	content.appendChild(createDivider());

	const ampSection = createAmpSection(callbacks, hasAmpCookie);
	content.appendChild(ampSection);

	content.appendChild(createDivider());

	root.appendChild(content);

	const aboutDivider = createDivider();
	root.appendChild(aboutDivider);

	const aboutSection = createAboutSection();
	root.appendChild(aboutSection);

	return root;
}

function createHeader(callbacks: SettingsCallbacks): HTMLElement {
	const header = document.createElement("div");
	header.className = "settings-header";

	const title = document.createElement("h2");
	title.className = "settings-title";
	title.textContent = "Settings";

	const closeButton = document.createElement("button");
	closeButton.className = "settings-close";
	closeButton.setAttribute("aria-label", "Close settings");

	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.setAttribute("width", "20");
	svg.setAttribute("height", "20");
	svg.setAttribute("viewBox", "0 0 24 24");
	svg.setAttribute("fill", "none");
	svg.setAttribute("stroke", "currentColor");
	svg.setAttribute("stroke-width", "2");
	svg.setAttribute("stroke-linecap", "round");
	svg.setAttribute("stroke-linejoin", "round");

	const line1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
	line1.setAttribute("x1", "18");
	line1.setAttribute("y1", "6");
	line1.setAttribute("x2", "6");
	line1.setAttribute("y2", "18");

	const line2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
	line2.setAttribute("x1", "6");
	line2.setAttribute("y1", "6");
	line2.setAttribute("x2", "18");
	line2.setAttribute("y2", "18");

	svg.appendChild(line1);
	svg.appendChild(line2);
	closeButton.appendChild(svg);

	closeButton.addEventListener("click", () => callbacks.onClose());

	header.appendChild(title);
	header.appendChild(closeButton);

	return header;
}

function createDivider(): HTMLElement {
	const divider = document.createElement("div");
	divider.className = "settings-divider";
	return divider;
}

function createZaiSection(callbacks: SettingsCallbacks, hasApiKey: boolean): HTMLElement {
	const section = document.createElement("div");
	section.className = "settings-section";

	const sectionTitle = document.createElement("div");
	sectionTitle.className = "settings-section-title";
	sectionTitle.textContent = "Z.AI Coding Plan API Key";

	section.appendChild(sectionTitle);

	if (hasApiKey) {
		section.appendChild(createZaiConnectedState(callbacks, section));
	} else {
		section.appendChild(createZaiInputState(callbacks, section));
	}

	return section;
}

function rebuildZaiSection(section: HTMLElement, callbacks: SettingsCallbacks, hasApiKey: boolean): void {
	const title = section.querySelector(".settings-section-title");
	while (section.lastChild) {
		section.removeChild(section.lastChild);
	}
	if (title) {
		section.appendChild(title);
	}

	if (hasApiKey) {
		section.appendChild(createZaiConnectedState(callbacks, section));
	} else {
		section.appendChild(createZaiInputState(callbacks, section));
	}
}

function createZaiConnectedState(callbacks: SettingsCallbacks, section: HTMLElement): HTMLElement {
	const container = document.createElement("div");
	container.className = "settings-credential-connected";

	const row = document.createElement("div");
	row.className = "settings-credential-row";

	const statusLeft = document.createElement("div");
	statusLeft.className = "settings-credential-status";

	const dot = document.createElement("span");
	dot.className = "gauge-dot status-success";

	const statusText = document.createElement("span");
	statusText.textContent = "API key configured";

	statusLeft.appendChild(dot);
	statusLeft.appendChild(statusText);

	const actions = document.createElement("div");
	actions.className = "settings-credential-actions";

	const updateButton = document.createElement("button");
	updateButton.className = "btn btn-ghost";
	updateButton.textContent = "Update";
	updateButton.addEventListener("click", () => {
		rebuildZaiSection(section, callbacks, false);
	});

	const removeButton = document.createElement("button");
	removeButton.className = "btn btn-destructive";
	removeButton.textContent = "Remove";
	removeButton.addEventListener("click", async () => {
		try {
			await callbacks.deleteZaiApiKey();
			await callbacks.onZaiKeyChanged();
			rebuildZaiSection(section, callbacks, false);
		} catch (error) {
			console.error("Failed to delete API key:", error);
		}
	});

	actions.appendChild(updateButton);
	actions.appendChild(removeButton);

	row.appendChild(statusLeft);
	row.appendChild(actions);
	container.appendChild(row);

	return container;
}

function createZaiInputState(callbacks: SettingsCallbacks, section: HTMLElement): HTMLElement {
	const container = document.createElement("div");
	container.className = "settings-credential-input";

	const desc = document.createElement("div");
	desc.className = "settings-credential-desc";
	desc.textContent = "Get your API key from ";

	const link = document.createElement("a");
	link.href = "javascript:void(0)";
	link.className = "zai-link";
	link.textContent = "z.ai/manage-apikey";
	link.addEventListener("click", async (e) => {
		e.preventDefault();
		const { invoke } = await import("@tauri-apps/api/core");
		await invoke("open_url", { url: "https://z.ai/manage-apikey/apikey-list" });
	});
	desc.appendChild(link);

	container.appendChild(desc);

	// Helper text for environment variable syntax
	const helperText = document.createElement("div");
	helperText.className = "settings-input-helper";
	helperText.textContent = "You can use {env:VAR} or $ENV:VAR";
	container.appendChild(helperText);

	const inputRow = document.createElement("div");
	inputRow.className = "settings-credential-input-row";

	const inputWrapper = document.createElement("div");
	inputWrapper.className = "settings-input-wrapper";

	const input = document.createElement("input");
	input.type = "password";
	input.placeholder = "Enter your API key";
	input.className = "zai-modal-input-field";

	// Toggle button to show/hide password (eye icon)
	const toggleButton = document.createElement("button");
	toggleButton.type = "button";
	toggleButton.className = "settings-toggle-visibility";
	toggleButton.setAttribute("aria-label", "Toggle password visibility");
	toggleButton.appendChild(createEyeIcon(false));

	toggleButton.addEventListener("click", () => {
		if (input.type === "password") {
			input.type = "text";
			toggleButton.replaceChildren(createEyeIcon(true));
		} else {
			input.type = "password";
			toggleButton.replaceChildren(createEyeIcon(false));
		}
	});

	inputWrapper.appendChild(input);
	inputWrapper.appendChild(toggleButton);

	// Save button
	const saveButton = document.createElement("button");
	saveButton.type = "button";
	saveButton.className = "btn btn-primary";
	saveButton.textContent = "Save";

	// Auto-switch to text when typing env var syntax
	input.addEventListener("input", () => {
		const value = input.value.trim();
		const isEnvVar = isEnvVarSyntax(value);

		if (isEnvVar) {
			input.type = "text";
			toggleButton.replaceChildren(createEyeIcon(true));
		} else {
			input.type = "password";
			toggleButton.replaceChildren(createEyeIcon(false));
		}
	});

	inputRow.appendChild(inputWrapper);
	inputRow.appendChild(saveButton);

	requestAnimationFrame(() => {
		input.focus();
	});

	const errorElement = document.createElement("div");
	errorElement.className = "zai-modal-error";
	errorElement.style.display = "none";

	const setValidationState = (validating: boolean, error?: string) => {
		saveButton.disabled = validating;
		saveButton.textContent = validating ? "Validating..." : "Save";
		input.disabled = validating;

		if (error) {
			errorElement.textContent = error;
			errorElement.style.display = "block";
		} else {
			errorElement.style.display = "none";
		}
	};

	saveButton.addEventListener("click", async () => {
		const apiKey = input.value.trim();
		if (!apiKey) {
			setValidationState(false, "Please enter an API key");
			return;
		}

		// Check if using environment variable syntax
		const isEnvVar = isEnvVarSyntax(apiKey);

		setValidationState(true);

		try {
			// Skip validation for environment variable syntax
			if (!isEnvVar) {
				await callbacks.validateZaiApiKey(apiKey);
			}
			await callbacks.saveZaiApiKey(apiKey);
			await callbacks.onZaiKeyChanged();
			rebuildZaiSection(section, callbacks, true);
		} catch (error) {
			setValidationState(false, String(error));
		}
	});

	input.addEventListener("keydown", (e) => {
		if (e.key === "Enter" && !saveButton.disabled) {
			saveButton.click();
		}
	});

	input.addEventListener("input", () => {
		if (errorElement.style.display !== "none") {
			setValidationState(false);
		}
	});

	container.appendChild(inputRow);
	container.appendChild(errorElement);

	return container;
}

function createAmpSection(callbacks: SettingsCallbacks, hasCookie: boolean): HTMLElement {
	const section = document.createElement("div");
	section.className = "settings-section";

	const sectionTitle = document.createElement("div");
	sectionTitle.className = "settings-section-title";
	sectionTitle.textContent = "AMP SESSION COOKIE";

	section.appendChild(sectionTitle);

	if (hasCookie) {
		section.appendChild(createAmpConnectedState(callbacks, section));
	} else {
		section.appendChild(createAmpInputState(callbacks, section));
	}

	return section;
}

function rebuildAmpSection(section: HTMLElement, callbacks: SettingsCallbacks, hasCookie: boolean): void {
	const title = section.querySelector(".settings-section-title");
	while (section.lastChild) {
		section.removeChild(section.lastChild);
	}
	if (title) {
		section.appendChild(title);
	}

	if (hasCookie) {
		section.appendChild(createAmpConnectedState(callbacks, section));
	} else {
		section.appendChild(createAmpInputState(callbacks, section));
	}
}

function createAmpConnectedState(callbacks: SettingsCallbacks, section: HTMLElement): HTMLElement {
	const container = document.createElement("div");
	container.className = "settings-credential-connected";

	const row = document.createElement("div");
	row.className = "settings-credential-row";

	const statusLeft = document.createElement("div");
	statusLeft.className = "settings-credential-status";

	const dot = document.createElement("span");
	dot.className = "gauge-dot status-success";

	const statusText = document.createElement("span");
	statusText.textContent = "Session cookie configured";

	statusLeft.appendChild(dot);
	statusLeft.appendChild(statusText);

	const actions = document.createElement("div");
	actions.className = "settings-credential-actions";

	const updateButton = document.createElement("button");
	updateButton.className = "btn btn-ghost";
	updateButton.textContent = "Update";
	updateButton.addEventListener("click", () => {
		rebuildAmpSection(section, callbacks, false);
	});

	const removeButton = document.createElement("button");
	removeButton.className = "btn btn-destructive";
	removeButton.textContent = "Remove";
	removeButton.addEventListener("click", async () => {
		try {
			await callbacks.deleteAmpSessionCookie();
			await callbacks.onAmpCookieChanged();
			rebuildAmpSection(section, callbacks, false);
		} catch (error) {
			console.error("Failed to delete session cookie:", error);
		}
	});

	actions.appendChild(updateButton);
	actions.appendChild(removeButton);

	row.appendChild(statusLeft);
	row.appendChild(actions);
	container.appendChild(row);

	return container;
}

function createAmpInputState(callbacks: SettingsCallbacks, section: HTMLElement): HTMLElement {
	const container = document.createElement("div");
	container.className = "settings-credential-input";

	const desc = document.createElement("div");
	desc.className = "settings-credential-desc";
	desc.textContent = "Copy your session cookie from ampcode.com. Open DevTools → Application → Cookies → session. ";

	const link = document.createElement("a");
	link.href = "javascript:void(0)";
	link.className = "zai-link";
	link.textContent = "ampcode.com/settings";
	link.addEventListener("click", async (e) => {
		e.preventDefault();
		await callbacks.openUrl("https://ampcode.com/settings");
	});
	desc.appendChild(link);

	container.appendChild(desc);

	const inputRow = document.createElement("div");
	inputRow.className = "settings-credential-input-row";

	const inputWrapper = document.createElement("div");
	inputWrapper.className = "settings-input-wrapper";

	const input = document.createElement("input");
	input.type = "password";
	input.placeholder = "Paste session cookie value";
	input.className = "zai-modal-input-field";

	const toggleButton = document.createElement("button");
	toggleButton.type = "button";
	toggleButton.className = "settings-toggle-visibility";
	toggleButton.setAttribute("aria-label", "Toggle cookie visibility");
	toggleButton.appendChild(createEyeIcon(false));

	toggleButton.addEventListener("click", () => {
		if (input.type === "password") {
			input.type = "text";
			toggleButton.replaceChildren(createEyeIcon(true));
		} else {
			input.type = "password";
			toggleButton.replaceChildren(createEyeIcon(false));
		}
	});

	inputWrapper.appendChild(input);
	inputWrapper.appendChild(toggleButton);

	const saveButton = document.createElement("button");
	saveButton.type = "button";
	saveButton.className = "btn btn-primary";
	saveButton.textContent = "Save";

	inputRow.appendChild(inputWrapper);
	inputRow.appendChild(saveButton);

	requestAnimationFrame(() => {
		input.focus();
	});

	const errorElement = document.createElement("div");
	errorElement.className = "zai-modal-error";
	errorElement.style.display = "none";

	saveButton.addEventListener("click", async () => {
		const cookie = input.value.trim();
		if (!cookie) {
			errorElement.textContent = "Please enter a session cookie";
			errorElement.style.display = "block";
			return;
		}

		saveButton.disabled = true;
		saveButton.textContent = "Validating...";
		input.disabled = true;

		try {
			await callbacks.validateAmpSessionCookie(cookie);
			saveButton.textContent = "Saving...";
			await callbacks.saveAmpSessionCookie(cookie);
			await callbacks.onAmpCookieChanged();
			rebuildAmpSection(section, callbacks, true);
		} catch (error) {
			errorElement.textContent = String(error);
			errorElement.style.display = "block";
			saveButton.disabled = false;
			saveButton.textContent = "Save";
			input.disabled = false;
		}
	});

	input.addEventListener("keydown", (e) => {
		if (e.key === "Enter" && !saveButton.disabled) {
			saveButton.click();
		}
	});

	input.addEventListener("input", () => {
		if (errorElement.style.display !== "none") {
			errorElement.style.display = "none";
		}
	});

	container.appendChild(inputRow);
	container.appendChild(errorElement);

	return container;
}

function createAboutSection(): HTMLElement {
	const section = document.createElement("div");
	section.className = "settings-section";

	const appName = document.createElement("div");
	appName.className = "settings-about-name";
	appName.textContent = "Usage Bar";

	const version = document.createElement("div");
	version.className = "settings-about-version";
	version.textContent = `Version ${APP_VERSION}`;

	section.appendChild(appName);
	section.appendChild(version);

	return section;
}
