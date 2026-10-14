#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QTcpSocket>
#include <QByteArray>
#include <QChart>
#include <QLineSeries>
#include <QValueAxis>
#include <QLineEdit>
#include <QSpinBox>
#include <QPushButton>
#include <QLabel>

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    void onConnected();
    void onReadyRead();
    void onDisconnected();

private:
    void sendCommand(const QString &json);
    void updateUIState(bool isRunning);

    QTcpSocket *m_socket;
    QByteArray m_buffer;

    QChart *m_chart;
    QLineSeries *m_rpsSeries;
    QValueAxis *m_axisX;
    QValueAxis *m_axisY;
    int m_xCounter;

    QLineEdit *m_urlEdit;
    QSpinBox *m_concurrentSpin;
    QSpinBox *m_durationSpin;
    QLabel *m_statusLabel;
};

#endif // MAINWINDOW_H