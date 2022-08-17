
var btn = document.getElementById("btn-dropdown");
var nav = document.getElementById("nav-dropdown");

btn.addEventListener('click', () => {
  if(!btn.classList.contains("is-open")) {
    btn.classList.add("is-open");
    nav.classList.add("is-open");
  } else {
    btn.classList.remove("is-open");
    nav.classList.remove("is-open");
  }
});


function loadJSON(callback) {
    var xObj = new XMLHttpRequest();
    xObj.overrideMimeType("application/json");
    xObj.open('GET', './data.json', true);
    xObj.onreadystatechange = function() {
        if (xObj.readyState === 4 && xObj.status === 200) {
            callback(xObj.responseText);
        }
    };
    xObj.send(null);
    return xObj.responseText;
}

function showInfo(timeFilter){
    btn.classList.remove("is-open");
    nav.classList.remove("is-open");

    document.body.classList.add('running');
    loadJSON(function(response){
        let json = JSON.parse(response);
        let info = getInfo(json, timeFilter);

        printRadialChart([info[0], info[1]], 'chart1');
        printRadialChart([info[0], info[2]], 'chart2');
        printLineChart(info[0], info[3], 'chart3');
        printLineChart(info[0], info[4], 'chart4');
        printLineChartTorrentsDates(info[5], 'chart5');
    });
}


function getInfo(trackerData, timeFilter){
    let torrentLabels = [];
    let nSeeders = [];
    let nLeechers = [];
    let activePeersInfo = [];
    let completedPeersInfo = [];
    let torrentDates = [];

    trackerData.torrents.forEach(torrent => {
        torrentLabels.push(torrent.name);
        nSeeders.push(torrent.seeders);
        nLeechers.push(torrent.leechers);
        torrentDates.push(torrent.timestamp);

        let taggedDatesActivePeers = getDatesActivePeers(torrent.peers, timeFilter);
        let initialNoOfActivePeers = getInitialNoOfActivePeers(torrent.peers, taggedDatesActivePeers[0][0]);

        let taggedDatesCompletedPeers = getDatesCompletedPeers(torrent.peers, timeFilter);
        let initialNoOfCompletedPeers = getInitialNoOfCompletedPeers(torrent.peers, taggedDatesCompletedPeers[0][0]);

        activePeersInfo.push(countPeers(taggedDatesActivePeers, initialNoOfActivePeers));
        completedPeersInfo.push(countPeers(taggedDatesCompletedPeers, initialNoOfCompletedPeers));

    });

    let torrentDatesInfo = getNoOfTorrentPerDate(torrentDates, timeFilter);
    return [torrentLabels, nSeeders, nLeechers, activePeersInfo, completedPeersInfo, torrentDatesInfo];
}

function getNoOfTorrentPerDate(torrentDates, timeFilter){
    let filteredDates = [];
    let firstDate = new Date(Date.now()-(timeFilter*1000)).toISOString().split('Z')[0];
    let lastDate = new Date(Date.now()).toISOString().split('Z')[0];
    torrentDates.push(firstDate);
    torrentDates.push(lastDate);

    torrentDates.sort(function(a,b){
        return convertToDate(a) - convertToDate(b);
    });

    for(i = 0; i < torrentDates.length; i++){
        if (i == torrentDates.length - 1){
            filteredDates.push({x:torrentDates[i], y:i-1});
        }else if(convertToDate(torrentDates[i]) >= convertToDate(firstDate)){
            filteredDates.push({x:torrentDates[i], y:i});
        }
    }

    return filteredDates;
}

function countPeers(taggedDates,initialNoOfPeers){
    let datesInfoPerTorrent = [];
    let dates = [];
    let peerCounter = [];
    peerCounter.push(initialNoOfPeers);

    for (let i = 0; i < taggedDates.length; i++){
        dates.push(taggedDates[i][0].split('+')[0]);
        if (i > 0){
            let previousCounter = peerCounter[i-1];
            if (taggedDates[i][1] == 'increase'){
                peerCounter.push(previousCounter + 1);
            }else if (taggedDates[i][1] == 'decrease'){
                peerCounter.push(previousCounter - 1);
            }else{
                peerCounter.push(previousCounter);
            }
        }
    }

    for (let j = 0; j < dates.length; j++){
        datesInfoPerTorrent.push({x:dates[j], y:peerCounter[j]});
    }

    return datesInfoPerTorrent;
}


function getDatesActivePeers(peers, timeFilter){
    let taggedDates = [];
    let firstDate = new Date(Date.now()-(timeFilter*1000)).toISOString().split('Z')[0];
    taggedDates.push([firstDate, "-"]);
    peers.forEach(peer => {
        let connection = convertToDate(peer.dt_connection);
        if ( (Date.now()/1000 - connection.getTime()/1000) <= timeFilter){
            taggedDates.push([peer.dt_connection, "increase"]);
        }

        if (peer.dt_disconnection != null){
            let disconnection = convertToDate(peer.dt_disconnection);
            if ( (Date.now()/1000 - disconnection.getTime()/1000) <= timeFilter){
                taggedDates.push([peer.dt_disconnection, "decrease"]);
            }
        }
    })

    taggedDates.sort(function(a,b){
        return convertToDate(a[0]) - convertToDate(b[0]);
    });

    taggedDates.push([new Date(Date.now()).toISOString().split('Z')[0],"-"]);
    return taggedDates;
}

function getDatesCompletedPeers(peers, timeFilter){
    let taggedDates = [];
    let firstDate = new Date(Date.now()-(timeFilter*1000)).toISOString().split('Z')[0];
    taggedDates.push([firstDate, "-"]);
    peers.forEach(peer => {
        if (peer.dt_completion != null){
            let completion = convertToDate(peer.dt_completion);
            if ( (Date.now()/1000 - completion.getTime()/1000) <= timeFilter){
                taggedDates.push([peer.dt_completion, "increase"]);
            }
            
            if (peer.dt_disconnection != null){
                let disconnection = convertToDate(peer.dt_disconnection);
                if ( (Date.now()/1000 - disconnection.getTime()/1000) <= timeFilter){
                    taggedDates.push([peer.dt_disconnection, "decrease"]);
                }
            }
        }

    })

    taggedDates.sort(function(a,b){
        return convertToDate(a[0]) - convertToDate(b[0]);
    });

    taggedDates.push([new Date(Date.now()).toISOString().split('Z')[0],"-"]);
    return taggedDates;
}

function getInitialNoOfActivePeers(peers, date){
    let peerCounter = 0;
    let initialDate = convertToDate(date);
    peers.forEach(peer =>{
        let connection = convertToDate(peer.dt_connection);
        if (connection <= initialDate){
            if (peer.dt_disconnection == null){
                peerCounter ++;
            }else if (convertToDate(peer.dt_disconnection) > initialDate){
                peerCounter ++;
            }
        }
    })

    return peerCounter;
}

function getInitialNoOfCompletedPeers(peers, date){
    let peerCounter = 0;
    let initialDate = convertToDate(date);
    peers.forEach(peer =>{
        if (peer.dt_completion != null){
            let completion = convertToDate(peer.dt_completion);
            if (completion <= initialDate){
                if (peer.dt_disconnection == null){
                    peerCounter ++;
                }else if (convertToDate(peer.dt_disconnection) > initialDate){
                    peerCounter ++;
                }
            }
        }
    })

    return peerCounter;
}



function convertToDate(dateString){
    let date = dateString.split('+')[0];
    return new Date(date+='Z');
}


function printRadialChart(info, chartId){
    const data = {
        labels: info[0],
        datasets: [
            {
                data: info[1],
                borderWidth: 1,
                borderColor: styles.color.solids.map(eachColor => eachColor),
                backgroundColor: styles.color.alphas.map(eachColor => eachColor)
            }
        ]
    }

    const options = {
        scale: {
            gridLines: {
                color: '#444'
            },
            ticks: {
                display: false
            }
        },
        legend: {
            position: 'bottom',
            labels: {
                fontColor: '#fff'
            }
        },
        maintainAspectRatio: false,
    }

    new Chart(chartId, { type: 'polarArea', data, options })
}



function printLineChart(labels, data, chartId){
    let allDatasets = [];

    for (let i = 0; i < labels.length; i++){
        let currDs = {
            label: labels[i],
            data: data[i],
            borderColor: styles.color.solids[i % styles.color.solids.length],
        };

        allDatasets.push(currDs);
    }

    new Chart(chartId, {
        type: 'line',
        data: { datasets: allDatasets },
        options: {
            scales: {
                xAxes: [{
                    type: 'time',
                    ticks: {
                        display: true,
                        frontColor: '#fff',
                        maxRotation: 90,
                        minRotation: 90
                    }
                }],
                yAxes: [{
                    ticks: {
                      stepSize: 1,
                      fixedStepSize: 1,
                      frontColor: '#fff'
                    }
                }],    
            },
            maintainAspectRatio: false,
            legend: {
                position: 'bottom',
                labels: {
                    fontColor: '#fff'
                }
            },
        }
    })
}

function printLineChartTorrentsDates(data, chartId){
    let currDs = {
        label:'Number of torrents',
        data,
        borderColor: styles.color.solids[0],
    };

    new Chart(chartId, {
        type: 'line',
        data: { datasets: [currDs] },
        options: {
            scales: {
                xAxes: [{
                    type: 'time',
                    ticks: {
                        display: true,
                        frontColor: '#fff',
                        maxRotation: 90,
                        minRotation: 90
                    }
                }],
                yAxes: [{
                    ticks: {
                      stepSize: 1,
                      fixedStepSize: 1,
                      frontColor: '#fff'
                    }
                }],    
            },
            maintainAspectRatio: false,
            legend: {
                position: 'bottom',
                labels: {
                    fontColor: '#fff'
                }
            },
        }
    })
}