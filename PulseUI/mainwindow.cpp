#include "mainwindow.h"
#include <QVBoxLayout>
#include <QChartView>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTimer>
#include <QPen>
#include <QPainter>

MainWindow::MainWindow(QWidget *parent) : QMainWindow(parent), m_xCounter(0) {
    resize(800, 600);
    setWindowTitle("PulseTest Monitor");

    m_rpsSeries = new QLineSeries();
    m_rpsSeries->setName("RPS");
    m_rpsSeries->setColor(Qt::cyan);
    m_rpsSeries->setPen(QPen(Qt::cyan, 2));

    m_chart = new QChart();
    m_chart->addSeries(m_rpsSeries);
    m_chart->setTitle("PulseTest - Ожидание данных...");
    m_chart->setTheme(QChart::ChartThemeDark);

    m_axisX = new QValueAxis();
    m_axisX->setRange(0, 50);
    m_axisX->setLabelFormat("%d");
    m_chart->addAxis(m_axisX, Qt::AlignBottom);
    m_rpsSeries->attachAxis(m_axisX);

    m_axisY = new QValueAxis();
    m_axisY->setRange(0, 200);
    m_chart->addAxis(m_axisY, Qt::AlignLeft);
    m_rpsSeries->attachAxis(m_axisY);

    QChartView *chartView = new QChartView(m_chart);
    chartView->setRenderHint(QPainter::Antialiasing);
    setCentralWidget(chartView);

    m_socket = new QTcpSocket(this);
    connect(m_socket, &QTcpSocket::connected, this, &MainWindow::onConnected);
    connect(m_socket, &QTcpSocket::readyRead, this, &MainWindow::onReadyRead);
    connect(m_socket, &QTcpSocket::disconnected, this, &MainWindow::onDisconnected);

    m_socket->connectToHost("127.0.0.1", 9000);
}

MainWindow::~MainWindow() {}

void MainWindow::onConnected() {
    m_chart->setTitle("PulseTest - Подключено к Rust-ядру");
    m_buffer.clear();
}

void MainWindow::onDisconnected() {
    m_chart->setTitle("PulseTest - Отключено. Переподключение...");
    QTimer::singleShot(2000, this, [this]() {
        m_socket->connectToHost("127.0.0.1", 9000);
    });
}

void MainWindow::onReadyRead() {
    m_buffer.append(m_socket->readAll());

    while (m_buffer.contains('\n')) {
        int newlineIndex = m_buffer.indexOf('\n');
        QByteArray line = m_buffer.left(newlineIndex);
        m_buffer.remove(0, newlineIndex + 1);

        if (line.isEmpty()) continue;

        QJsonParseError error;
        QJsonDocument doc = QJsonDocument::fromJson(line, &error);
        if (error.error == QJsonParseError::NoError && doc.isObject()) {
            QJsonObject obj = doc.object();
            
            if (obj.contains("rps")) {
                double rps = obj["rps"].toDouble();
                
                m_rpsSeries->append(m_xCounter, rps);
                m_xCounter++;

                if (m_xCounter > 50) {
                    m_axisX->setRange(m_xCounter - 50, m_xCounter);
                }

                double maxY = 0;
                for (const QPointF &point : m_rpsSeries->points()) {
                    if (point.y() > maxY) maxY = point.y();
                }
                
                if (maxY > 0) {
                    m_axisY->setRange(0, maxY * 1.2 + 10);
                }

                QString title = QString("RPS: %1 | Avg: %2 ms | P95: %3 ms")
                    .arg(obj["rps"].toInt())
                    .arg(obj["avg_latency_ms"].toString())
                    .arg(obj["p95_latency_ms"].toString());
                m_chart->setTitle(title);
            }
        }
    }
}