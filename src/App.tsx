import type { Component } from 'solid-js';
import BodyGroupsMenu from './components/bodygroups/BodyGroupsMenu';

const App: Component = () => {
    return (
        <>
            <header>
                <h1>Source Wrench</h1>
                <nav>
                    <ul>
                        <li>
                            <a href="#bodygroups">Body Groups</a>
                        </li>
                    </ul>
                </nav>
            </header>

            <main>
                <BodyGroupsMenu id="bodygroups" />
            </main>
        </>
    );
};

export default App;
