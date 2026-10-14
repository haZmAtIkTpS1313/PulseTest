cmake_minimum_required(VERSION 3.16)
project(PulseUI LANGUAGES CXX)

list(APPEND CMAKE_PREFIX_PATH "/usr/lib64/qt6")
list(APPEND CMAKE_PREFIX_PATH "/usr")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)


find_package(Qt6 REQUIRED COMPONENTS Widgets Network Charts)

qt_standard_project_setup()

qt_add_executable(PulseUI
    main.cpp
    mainwindow.h
    mainwindow.cpp
)

target_link_libraries(PulseUI PRIVATE
    Qt6::Widgets
    Qt6::Network
    Qt6::Charts
)