class App {
    events: any[] | null;

    constructor() {
        this.events = null;
    }

    setEvents(jsonText: string) {
        const json = JSON.parse(jsonText);
        this.events = json;
        this.displayEvents(this.events);

        const eventTypes: string[] = [];

        for (const event of this.events) {
            const tp = event.detected_event.type as string;
            if (!eventTypes.includes(tp)) {
                eventTypes.push(tp);
            }
        }

        const typeSelect = document.getElementById('eventTypeSelect') as HTMLSelectElement;
        typeSelect.innerHTML = "";
        const anyOption = document.createElement('option');
        anyOption.id = 'any';
        anyOption.text = '------------';
        typeSelect.add(anyOption);
        for (const tp of eventTypes) {
            const option = document.createElement('option');
            option.id = tp;
            option.text = tp;
            typeSelect.add(option);
        }
    }

    filterEventsByType(typeFilter: string) {
        if (typeFilter === 'any') {
            this.displayEvents(this.events);
            return;
        }

        const newList = [];
        for (const event of this.events) {
            if (event.detected_event.type === typeFilter) {
                newList.push(event);
            }
        }

        this.displayEvents(newList);
    }

    displayEvents(events: any[]) {
        const eventTypes: string[] = [];

        document.getElementById('list').innerHTML = "";
        for (const event of events) {
            const timestamp = new Date(event.timestamp);
            const tp = event.detected_event.type as string;
            const id = event.id;

            if (!eventTypes.includes(tp)) {
                eventTypes.push(tp);
            }

            const container = document.createElement('div');
            container.classList.add('listItem');
            container.setAttribute('id', id);
            const header = document.createElement('div');
            header.classList.add('itemHeader');
            const timestampContainer = document.createElement('p');
            timestampContainer.classList.add('timestamp');
            timestampContainer.innerHTML = App.customDateTimeString(timestamp);
            const typeContainer = document.createElement('p');
            typeContainer.innerHTML = App.cleanString(tp);
            const detailsContainer = document.createElement('div');
            detailsContainer.style['marginLeft'] = '10px';
            detailsContainer.style['display'] = 'none';

            container.appendChild(header);
            header.appendChild(timestampContainer);
            header.appendChild(typeContainer);
            container.appendChild(detailsContainer);

            App.displayDetails(detailsContainer, event.detected_event.content);

            document.getElementById('list').appendChild(container);

            container.addEventListener('click', () => {
                console.log("a");
                if (detailsContainer.style['display'] == 'none') {
                    detailsContainer.style['display'] = 'block';
                } else {
                    detailsContainer.style['display'] = 'none';
                }
            });
        }
        document.getElementById('eventsCount').innerHTML = this.events.length.toString();
        document.getElementById('eventsSelectedCount').innerHTML = events.length.toString();
    }

    static displayDetails(container: HTMLDivElement, content: any) {
        for (let [key, value] of Object.entries(content)) {
            if (!value) {
                continue;
            }
            const detailContainer = document.createElement('div');
            const pairContainer = document.createElement('div');
            pairContainer.classList.add('detailKeyValue');
            const keyContainer = document.createElement('p');
            keyContainer.innerHTML = App.cleanString(key);
            keyContainer.style['marginRight'] = '10px';
            keyContainer.style['fontWeight'] = 'bold';
            const valueContainer = document.createElement('p');

            container.appendChild(detailContainer);
            detailContainer.appendChild(pairContainer);
            pairContainer.appendChild(keyContainer);
            pairContainer.appendChild(valueContainer);

            if (typeof value == 'object') {
                const subContainer = document.createElement('div');
                subContainer.style['marginLeft'] = '10px';
                this.displayDetails(subContainer, value);
                detailContainer.appendChild(subContainer);
            } else {
                valueContainer.innerHTML = value.toString();
            }
        }
    }

    static customDateTimeString(date: Date): string {
        const year = date.getFullYear().toString();
        const day = date.getDate().toString().padStart(2, '0');
        const month = (date.getMonth() + 1).toString().padStart(2, '0');
        const hour = date.getHours().toString().padStart(2, '0');
        const minute = date.getMinutes().toString().padStart(2, '0');
        const second = date.getSeconds().toString().padStart(2, '0');
        const millisecond = date.getMilliseconds().toString().padStart(3, '0');

        return `${year}-${month}-${day}, ${hour}:${minute}:${second}.${millisecond}`
    }

    static cleanString(str: string): string {
        const withSpaces = str.replace(new RegExp(escapeRegExp('_'), 'g'), ' ');
        return withSpaces.charAt(0).toUpperCase() + withSpaces.slice(1);
    }
}

function escapeRegExp(str: string) {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); // $& means the whole matched string
}

const app = new App();


window.addEventListener('load', () => {
    document.getElementById('body').addEventListener('dragenter', (event) => {
        event.preventDefault();
    });

    document.getElementById('body').addEventListener('dragover', (event) => {
        event.preventDefault();
    });

    document.getElementById('body').addEventListener('drop', (event) => {
        event.preventDefault();
    
        if (event.dataTransfer.items) {
            for (const item of event.dataTransfer.items) {
                if (item.kind == 'file') {
                    const file = item.getAsFile();
                        file.text().then((text) => {
                            app.setEvents(text);
                        }
                    );
                }
            }
        }
    });
    
    const options = document.getElementById('eventTypeSelect') as HTMLSelectElement;
    options.addEventListener('change', () => {
        const i = options.selectedIndex;
        app.filterEventsByType(options.options[i].id);
    });
});