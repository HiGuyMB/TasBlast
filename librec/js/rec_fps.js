function readFileAsync(file) {
    return new Promise((resolve, reject) => {
        let reader = new FileReader();

        reader.onload = () => {
            resolve(reader.result);
        };

        reader.onerror = reject;

        reader.readAsArrayBuffer(file);
    })
}

import("../pkg/index.js").then(async (librec) => {
    let conts = await (await fetch("template/rec.hbs")).text();
    let template = Handlebars.compile(conts);

    const output_div = document.getElementById("output_text");
    const input_file = document.getElementById("input_file");

    input_file.addEventListener('change', async (e) => {
        let conts = input_file.files[0];
        if (conts) {
            let result = await readFileAsync(conts);
            let converted = librec.import_json(new Uint8Array(result));
            try {
                let parsed = JSON.parse(converted);
                parseRec(parsed);
            } catch (e) {
                output_div.innerText = "Error: " + e.toString();
            }
        }
    });

    function parseRec(converted) {
        /*
        converted: {
            mission: "something",
            frames: [
                {
                    dt: 1
                    moves: [
                        {
                            yaw: 0,
                            pitch: 0,
                            roll: 0,
                            mx: 0,
                            my: 0,
                            mz: 0,
                            freelook: [false, false, false, false, false, false]
                        },
                        null
                    ]
                }
            ]
        }
        */
        let max_time = 0;
        for (let i = 0; i < converted.frames.length; i ++) {
            let frame = converted.frames[i];
            max_time += frame.delta;
        }

        let fps_window = 10;
        function getFPS(time, i, fps_window) {
            let neg_time = time + converted.frames[i].delta;
            let pos_time = time;

            let neg_frames = 0;
            for (let j = 0; j < fps_window; j ++) {
                if (j > i) {
                    break;
                }
                neg_time -= converted.frames[i - j].delta;
                neg_frames ++;
            }
            let pos_frames = 0;
            for (let j = 0; j < fps_window; j ++) {
                if (i + j >= converted.frames.length) {
                    break;
                }
                pos_time += converted.frames[i + j].delta;
                pos_frames ++;
            }

            return (neg_frames + pos_frames - 1) / (pos_time - neg_time) * 1000.0;
        }

        let times = [];
        let fpses = [];
        let deltas = [];
        let time = 0;
        times.push(time);
        fpses.push(getFPS(time, 0, fps_window));
        for (let i = 0; i < converted.frames.length; i ++) {
            let frame = converted.frames[i];
            time += frame.delta;
            times.push(time);
            fpses.push(getFPS(time, i, fps_window));
            deltas.push(frame.delta);
        }
        output_div.innerHTML = template(converted);

        Plotly.newPlot("rec-ms-plot", [{
            x: times,
            y: deltas
        }], {
            margin: {
                t: 0
            },
            xaxis: {
                title: "Time",
                range: [times[0], times[times.length - 1]]
            },
            yaxis: {
                title: "DT",
                range: [0, Math.max(...deltas)]
            },
            textfont: {
                color: '#ffffff'
            },
            fillcolor: 'rgb(0, 0, 0)'
        });
        Plotly.newPlot("rec-fps-plot", [{
            x: times,
            y: fpses
        }], {
            margin: {
                t: 0
            },
            xaxis: {
                title: "Time",
                range: [times[0], times[times.length - 1]]
            },
            yaxis: {
                title: "FPS",
                range: [0, Math.max(...fpses)]
            },
            textfont: {
                color: '#ffffff'
            },
            fillcolor: 'rgb(0, 0, 0)'
        });
    }

    function downloadBlob(data, fileName, mimeType) {
        let blob = new Blob([data], {
            type: mimeType
        });
        let url = window.URL.createObjectURL(blob);
        downloadURL(url, fileName);
        setTimeout(function() {
            return window.URL.revokeObjectURL(url);
        }, 1000);
    }

    function downloadURL(data, fileName) {
        let a = document.createElement('a');
        a.href = data;
        a.download = fileName;
        document.body.appendChild(a);
        a.style = 'display: none';
        a.click();
        a.remove();
    }
}).catch(console.error)