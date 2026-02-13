let modalElement: HTMLElement | null = null;

// Callbacks for Z.ai operations - will be attached by main.ts
let checkZaiApiKeyCallback: (() => Promise<boolean>) | null = null;
let validateZaiApiKeyCallback: ((apiKey: string) => Promise<void>) | null = null;
let saveZaiApiKeyCallback: ((apiKey: string) => Promise<void>) | null = null;
let deleteZaiApiKeyCallback: (() => Promise<void>) | null = null;
let refreshZaiUICallback: (() => Promise<void>) | null = null;

export function setZaiCallbacks(callbacks: {
	checkZaiApiKey: () => Promise<boolean>;
	validateZaiApiKey: (apiKey: string) => Promise<void>;
	saveZaiApiKey: (apiKey: string) => Promise<void>;
	deleteZaiApiKey: () => Promise<void>;
	refreshZaiUI: () => Promise<void>;
}): void {
	checkZaiApiKeyCallback = callbacks.checkZaiApiKey;
	validateZaiApiKeyCallback = callbacks.validateZaiApiKey;
	saveZaiApiKeyCallback = callbacks.saveZaiApiKey;
	deleteZaiApiKeyCallback = callbacks.deleteZaiApiKey;
	refreshZaiUICallback = callbacks.refreshZaiUI;
}

export function openZaiModal(isConnected: boolean): void {
	openModal(isConnected);
}

export async function checkZaiApiKey(): Promise<boolean> {
	if (!checkZaiApiKeyCallback) {
		throw new Error("checkZaiApiKeyCallback not set");
	}
	return await checkZaiApiKeyCallback();
}

export function createZaiConnectionBadge(isConnected: boolean): HTMLElement {
	const container = document.createElement("button");
	container.className = isConnected ? "zai-header-badge zai-header-badge-connected" : "zai-header-badge zai-header-badge-disconnected";

	const icon = document.createElement("span");
	icon.className = "zai-header-badge-icon";

	if (isConnected) {
		const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
		svg.setAttribute("width", "12");
		svg.setAttribute("height", "12");
		svg.setAttribute("viewBox", "0 0 24 24");
		svg.setAttribute("fill", "none");
		svg.setAttribute("stroke", "currentColor");
		svg.setAttribute("stroke-width", "3");
		svg.setAttribute("stroke-linecap", "round");
		svg.setAttribute("stroke-linejoin", "round");
		const polyline = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
		polyline.setAttribute("points", "20 6 9 17 4 12");
		svg.appendChild(polyline);
		icon.appendChild(svg);
	} else {
		const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
		svg.setAttribute("width", "12");
		svg.setAttribute("height", "12");
		svg.setAttribute("viewBox", "0 0 24 24");
		svg.setAttribute("fill", "none");
		svg.setAttribute("stroke", "currentColor");
		svg.setAttribute("stroke-width", "2");
		svg.setAttribute("stroke-linecap", "round");
		svg.setAttribute("stroke-linejoin", "round");
		const line1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
		line1.setAttribute("x1", "12");
		line1.setAttribute("y1", "5");
		line1.setAttribute("x2", "12");
		line1.setAttribute("y2", "19");
		const line2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
		line2.setAttribute("x1", "5");
		line2.setAttribute("y1", "12");
		line2.setAttribute("x2", "19");
		line2.setAttribute("y2", "12");
		svg.appendChild(line1);
		svg.appendChild(line2);
		icon.appendChild(svg);
	}

	const label = document.createElement("span");
	label.className = "zai-header-badge-label";
	label.textContent = isConnected ? "Connected" : "Connect";

	container.appendChild(icon);
	container.appendChild(label);

	container.addEventListener("click", () => openModal(isConnected));

	return container;
}

export async function createZaiSettings(): Promise<HTMLElement> {
	const settings = document.createElement("div");
	settings.className = "zai-settings";

	const hasApiKey = await checkZaiApiKey();

	if (!hasApiKey) {
		settings.style.display = "none";
	}

	return settings;
}

function openModal(isConnected: boolean): void {
	if (modalElement) return;

	const backdrop = document.createElement("div");
	backdrop.className = "modal-backdrop";

	modalElement = document.createElement("div");
	modalElement.className = "zai-modal";

	const header = document.createElement("div");
	header.className = "zai-modal-header";

	const title = document.createElement("h2");
	title.className = "zai-modal-title";
	title.textContent = isConnected ? "Manage z.ai API Key" : "Connect z.ai Coding Plan";

	const closeButton = document.createElement("button");
	closeButton.className = "zai-modal-close";
	closeButton.setAttribute("aria-label", "Close modal");
	const closeSvg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	closeSvg.setAttribute("width", "20");
	closeSvg.setAttribute("height", "20");
	closeSvg.setAttribute("viewBox", "0 0 24 24");
	closeSvg.setAttribute("fill", "none");
	closeSvg.setAttribute("stroke", "currentColor");
	closeSvg.setAttribute("stroke-width", "2");
	closeSvg.setAttribute("stroke-linecap", "round");
	closeSvg.setAttribute("stroke-linejoin", "round");
	const closeLine1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
	closeLine1.setAttribute("x1", "18");
	closeLine1.setAttribute("y1", "6");
	closeLine1.setAttribute("x2", "6");
	closeLine1.setAttribute("y2", "18");
	const closeLine2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
	closeLine2.setAttribute("x1", "6");
	closeLine2.setAttribute("y1", "6");
	closeLine2.setAttribute("x2", "18");
	closeLine2.setAttribute("y2", "18");
	closeSvg.appendChild(closeLine1);
	closeSvg.appendChild(closeLine2);
	closeButton.appendChild(closeSvg);

	header.appendChild(title);
	header.appendChild(closeButton);

	const content = document.createElement("div");
	content.className = "zai-modal-content";

	if (isConnected) {
		content.appendChild(createConnectedState());
	} else {
		content.appendChild(createInputState());
	}

	modalElement.appendChild(header);
	modalElement.appendChild(content);

	backdrop.appendChild(modalElement);
	document.body.appendChild(backdrop);

	backdrop.addEventListener("click", (e) => {
		if (e.target === backdrop) closeModal();
	});

	closeButton.addEventListener("click", (e) => {
		e.stopPropagation();
		closeModal();
	});
}

function closeModal(): void {
	if (!modalElement) return;

	const backdrop = modalElement.parentElement;
	if (backdrop) {
		// Prevent any pointer events during close animation
		backdrop.style.pointerEvents = "none";
		backdrop.style.animation = "fade-out 0.15s ease-out forwards";

		requestAnimationFrame(() => {
			setTimeout(() => {
				backdrop.remove();
				modalElement = null;
			}, 150);
		});
	}
}

function createConnectedState(): HTMLElement {
	const container = document.createElement("div");
	container.className = "zai-modal-connected";

	const info = document.createElement("div");
	info.className = "zai-modal-info";

	const infoText = document.createElement("div");
	infoText.className = "zai-modal-info-text";

	const infoTitle = document.createElement("div");
	infoTitle.className = "zai-modal-info-title";
	infoTitle.textContent = "API Key Configured";

	const infoDesc = document.createElement("div");
	infoDesc.className = "zai-modal-info-desc";
	infoDesc.textContent = "Your Z.ai API key is set up and ready to use.";

	infoText.appendChild(infoTitle);
	infoText.appendChild(infoDesc);
	info.appendChild(infoText);

	const actions = document.createElement("div");
	actions.className = "zai-modal-actions";

	const cancelButton = document.createElement("button");
	cancelButton.className = "btn btn-ghost";
	cancelButton.textContent = "Cancel";
	cancelButton.addEventListener("click", closeModal);

	const updateButton = document.createElement("button");
	updateButton.className = "btn btn-primary";
	updateButton.textContent = "Update";
	updateButton.addEventListener("click", () => {
		const content = modalElement?.querySelector(".zai-modal-content");
		if (content) {
			content.innerHTML = "";
			content.appendChild(createInputState());
		}
	});

	const deleteButton = document.createElement("button");
	deleteButton.className = "btn btn-destructive";
	deleteButton.textContent = "Remove";
	deleteButton.addEventListener("click", async () => {
		try {
			if (!deleteZaiApiKeyCallback) {
				throw new Error("deleteZaiApiKeyCallback not set");
			}
			await deleteZaiApiKeyCallback();
			closeModal();
			if (refreshZaiUICallback) {
				await refreshZaiUICallback();
			} else {
				throw new Error("refreshZaiUICallback not set");
			}
		} catch (error) {
			console.error("Failed to delete API key:", error);
			// Show error in modal instead of closing
			const errorDiv = document.createElement("div");
			errorDiv.className = "zai-modal-error";
			errorDiv.textContent = String(error);
			errorDiv.style.display = "block";

			// Add error to the modal content
			const content = modalElement?.querySelector(".zai-modal-content");
			if (content) {
				// Remove any existing error
				const existingError = content.querySelector(".zai-modal-error");
				if (existingError) {
					existingError.remove();
				}
				content.appendChild(errorDiv);
			}
		}
	});

	actions.appendChild(updateButton);
	actions.appendChild(deleteButton);
	actions.appendChild(cancelButton);

	container.appendChild(info);
	container.appendChild(actions);

	return container;
}

function createInputState(): HTMLElement {
	const container = document.createElement("div");
	container.className = "zai-modal-input";

	const info = document.createElement("div");
	info.className = "zai-modal-info";

	const infoText = document.createElement("div");
	infoText.className = "zai-modal-info-text";

	const infoTitle = document.createElement("div");
	infoTitle.className = "zai-modal-info-title";
	infoTitle.textContent = "Enter Your API Key";

	const infoDesc = document.createElement("div");
	infoDesc.className = "zai-modal-info-desc";
	infoDesc.textContent = "Get your API key from ";
	const link = document.createElement("a");
	link.href = "https://z.ai/manage-apikey/apikey-list";
	link.target = "_blank";
	link.className = "zai-link";
	link.textContent = "z.ai/manage-apikey";
	infoDesc.appendChild(link);

	infoText.appendChild(infoTitle);
	infoText.appendChild(infoDesc);
	info.appendChild(infoText);

	const inputContainer = document.createElement("div");
	inputContainer.className = "zai-modal-input-container";

	const input = document.createElement("input");
	input.type = "password";
	input.placeholder = "Enter your API key";
	input.className = "zai-modal-input-field";
	input.autofocus = true;

	const errorElement = document.createElement("div");
	errorElement.className = "zai-modal-error";
	errorElement.style.display = "none";

	const actions = document.createElement("div");
	actions.className = "zai-modal-actions";

	const saveButton = document.createElement("button");
	saveButton.className = "btn btn-primary";
	saveButton.textContent = "Save";

	const cancelButton = document.createElement("button");
	cancelButton.className = "btn btn-ghost";
	cancelButton.textContent = "Cancel";
	cancelButton.addEventListener("click", closeModal);

	const setValidationState = (validating: boolean, error?: string) => {
		saveButton.disabled = validating;
		saveButton.textContent = validating ? "Validating..." : "Save";
		input.disabled = validating;
		cancelButton.disabled = validating;

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
			if (!validateZaiApiKeyCallback || !saveZaiApiKeyCallback) {
				throw new Error("Zai API callbacks not set");
			}
			await validateZaiApiKeyCallback(apiKey);
			await saveZaiApiKeyCallback(apiKey);
			closeModal();
			if (refreshZaiUICallback) {
				await refreshZaiUICallback();
			} else {
				throw new Error("refreshZaiUICallback not set");
			}
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

	inputContainer.appendChild(input);
	inputContainer.appendChild(errorElement);
	actions.appendChild(saveButton);
	actions.appendChild(cancelButton);

	container.appendChild(info);
	container.appendChild(inputContainer);
	container.appendChild(actions);

	return container;
}

