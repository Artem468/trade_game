import {useEffect, useState} from 'react'
import Chart from "react-apexcharts";
import ru from "apexcharts/dist/locales/ru.json"
import {useSearchParams} from "react-router-dom"
import {ApexOptions} from "apexcharts";
import './App.css'

interface PriceHistory {
    status: string,
    data: PriceItem[]
    error: string | null
}

interface PriceItem {
    price: string,
    timestamp: number
}

function App() {
    let [searchParams, _setSearchParams] = useSearchParams()
    const [graphData, setGraphData] = useState<PriceItem[]>([])

    function _getData() {
        fetch(`../api/v1/price/history/${searchParams.get("asset_id")}`, {
            method: "GET"
        })
            .then((response) => response.json())
            .then((data: PriceHistory) => {
                setGraphData(data.data);
            })
            .catch((error) => console.log(error));
    }

    useEffect(() => {
        _getData()
        const interval = setInterval(() => {
            _getData()
        }, 60000);

        return () => clearInterval(interval);
    }, []);

    const chartOptions: ApexOptions = {
        chart: {
            fontFamily: `inherit`,
            background: searchParams.get("is_dark") === "true" ? "#212121": "#ffffff",
            zoom: {
                enabled: false,
            },
            toolbar: {
                show: false,
            },
            locales: [ru],
            defaultLocale: 'ru',
        },
        dataLabels: {
            enabled: false,
        },
        fill: {
            type: "gradient",
            gradient: {
                shadeIntensity: 0,
                inverseColors: false,
                opacityFrom: 0.2,
                stops: [100],
            },
        },
        stroke: {
            width: 3,
            curve: "smooth",
        },
        colors: [
            "#1A759F",
        ],
        xaxis: {
            type: "datetime",
            axisBorder: {
                color: searchParams.get("is_dark") === "true" ? "#adb0bb" : "#212121",
            },
            labels: {
                datetimeUTC: false,
                show: false
            }
        },
        yaxis: {
            opposite: false,
            labels: {
                show: true,
            },
        },
        legend: {
            show: false,
        },
        grid: {
            show: false,
        },
        tooltip: {
            theme: "dark",
            fillSeriesColor: false,
            x: {
                format: "HH:mm",
            },
        },
    };

    function dataLineToView() {
        const groupedData: Record<string, { x: number; y: number }[]> = {};

        graphData.forEach(({price, timestamp}: PriceItem) => {
            if (!groupedData["Цена"]) {
                groupedData["Цена"] = [];
            }
            groupedData["Цена"].push({x: timestamp * 1000, y: Number(price)});
        });

        return Object.entries(groupedData)
            .map(([name, data]) => ({name, data}));
    }

    console.log(dataLineToView())

    return (
        <>
            <Chart
                options={chartOptions}
                series={dataLineToView()}
                type="area"
                height="450px"
                width="100%"
            />
        </>
    )
}

export default App
