/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2025 MNX Cloud, Inc.
 * Copyright 2025 Edgecast Cloud LLC.
 */

@Library('jenkins-joylib@v1.0.8') _

pipeline {

    agent {
        label joyCommonLabels(image_ver: '24.4.1')
    }

    options {
        buildDiscarder(logRotator(numToKeepStr: '30'))
        timestamps()
    }

    stages {
        stage('check') {
            steps{
                sh('make check')
            }
        }
        stage('test') {
            steps{
                sh('make test')
            }
        }
        stage('make publish') {
            steps {
                sh('make publish bits-upload')
            }
        }
    }

    post {
        always {
            joySlackNotifications()
            joySlackNotifications(channel: 'cloud-smartos')
        }
    }
}
