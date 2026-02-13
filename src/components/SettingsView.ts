export interface SettingsCallbacks {
	checkZaiApiKey: () => Promise<boolean>;
	validateZaiApiKey: (apiKey: string) => Promise<void>;
	saveZaiApiKey: (apiKey: string) => Promise<void>;
	deleteZaiApiKey: () => Promise<void>;
	onZaiKeyChanged: () => Promise<void>;
	onClose: () => void;
}

export function createSettingsView(callbacks: SettingsCallbacks, hasZaiApiKey: boolean): HTMLElement {
	const root = document.createElement("div");
	root.id = "settings-view";
	root.className = "settings-view";

	root.appendChild(createHeader(callbacks));

	const content = document.createElement("div");
	content.className = "settings-content";

	const zaiSection = createZaiSection(callbacks, hasZaiApiKey);
	content.appendChild(zaiSection);

	content.appendChild(createDivider());
	content.appendChild(createAboutSection());

	root.appendChild(content);

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
	sectionTitle.textContent = "Z.ai API Key";

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
	container.className = "settings-zai-connected";

	const row = document.createElement("div");
	row.className = "settings-zai-row";

	const statusLeft = document.createElement("div");
	statusLeft.className = "settings-zai-status";

	const dot = document.createElement("span");
	dot.className = "gauge-dot status-success";

	const statusText = document.createElement("span");
	statusText.textContent = "API key configured";

	statusLeft.appendChild(dot);
	statusLeft.appendChild(statusText);

	const actions = document.createElement("div");
	actions.className = "settings-zai-actions";

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
	container.className = "settings-zai-input";

	const desc = document.createElement("div");
	desc.className = "settings-zai-desc";
	desc.textContent = "Get your API key from ";

	const link = document.createElement("a");
	link.href = "#";
	link.className = "zai-link";
	link.textContent = "z.ai/manage-apikey";
	link.addEventListener("click", async (e) => {
		e.preventDefault();
		const { openUrl } = await import("@tauri-apps/plugin-opener");
		await openUrl("https://z.ai/manage-apikey/apikey-list");
	});
	desc.appendChild(link);

	const inputRow = document.createElement("div");
	inputRow.className = "settings-zai-input-row";

	const input = document.createElement("input");
	input.type = "password";
	input.placeholder = "Enter your API key";
	input.className = "zai-modal-input-field";
	input.autofocus = true;

	const saveButton = document.createElement("button");
	saveButton.className = "btn btn-primary";
	saveButton.textContent = "Save";

	inputRow.appendChild(input);
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

		setValidationState(true);

		try {
			await callbacks.validateZaiApiKey(apiKey);
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

	container.appendChild(desc);
	container.appendChild(inputRow);
	container.appendChild(errorElement);

	return container;
}

function createAboutSection(): HTMLElement {
	const section = document.createElement("div");
	section.className = "settings-section";

	const sectionTitle = document.createElement("div");
	sectionTitle.className = "settings-section-title";
	sectionTitle.textContent = "About";

	const appName = document.createElement("div");
	appName.className = "settings-about-name";
	appName.textContent = "Usage Bar";

	const version = document.createElement("div");
	version.className = "settings-about-version";
	version.textContent = "Version 1.0.0";

	section.appendChild(sectionTitle);
	section.appendChild(appName);
	section.appendChild(version);

	return section;
}
